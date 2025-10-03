use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use error_stack::{Report, ResultExt};
use http::Method;
use http_msgsign_draft::digest::body::Body;
use http_msgsign_draft::digest::Digest;
use http_msgsign_draft::errors::SignatureInputError;
use http_msgsign_draft::sign::{RequestSign, ResponseSign, SignatureParams};
use http_msgsign_draft::sign::headers::SignatureInput;
use serde::{Deserialize, Serialize};
use kernel::entities::activity::Activity;
use kernel::entities::links::types::PublicKey;

use crate::config::Config;
use crate::error::{InquiryError, SetupError, TransportError, VerificationError};
use crate::hasher::Sha256Hasher;
use crate::keyset::{RsaSignerKey, RsaVerifierKey};

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    signer: Arc<RsaSignerKey>,
    authority_overrides: HashMap<String, String>
}

static SIGNATURE_PARAMS: LazyLock<SignatureParams> = LazyLock::new(|| {
    SignatureParams::builder()
        .add_request_target()
        .add_header("host")
        .add_header("date")
        .add_header("digest")
        .add_header("content-type")
        .build()
        .unwrap()
});

impl HttpClient {
    #[tracing::instrument(skip_all)]
    pub fn setup(config: Config) -> Result<Self, Report<SetupError>> {
        let mut client = reqwest::Client::builder();
        let mut authority_overrides = HashMap::new();
        
        for (host, overrides) in config.server.overrides {
            if let Some(cert) = overrides.certificate {
                let cert = reqwest::Certificate::from_pem(
                    std::fs::read(&cert)
                        .change_context_lazy(|| SetupError)?
                        .as_slice()
                ).change_context_lazy(|| SetupError)?;
                
                tracing::debug!("Add custom root certificate for {host}.");
                client = client.add_root_certificate(cert);
            }
            
            if let Some(authority) = overrides.authority {
                tracing::debug!("Requests from {host} will have their `authority` changed to {authority}.");
                authority_overrides.insert(host, authority);
            }
        }
        
        let client = client.build()
            .change_context_lazy(|| SetupError)?;
        
        let signer = RsaSignerKey::load(
            config.server.host_name,
            "relay.actor".to_string(),
            config.server.host_key
        ).change_context_lazy(|| SetupError)?;
        
        Ok(Self {
            client,
            signer: Arc::new(signer),
            authority_overrides
        })
    }
    
    pub async fn send_activity(&self, uri: impl AsRef<str>, activity: &Activity) -> Result<(), Report<TransportError>> {
        let body = serde_json::to_vec(activity)
            .change_context_lazy(|| TransportError::Serialization)?;
        
        let uri = uri.as_ref().parse::<http::Uri>()
            .change_context_lazy(|| TransportError::Request)?;
        
        let Some(authority) = uri.authority().map(ToString::to_string) else {
            return Err(Report::new(TransportError::Request)
                .attach("URI must have an authority."));
        };
        
        let mut uri = uri.into_parts();
        
        if let Some(new_authority) = self.authority_overrides.get(&authority) {
            uri.authority = Some(new_authority.parse().change_context_lazy(|| TransportError::Request)?);
        }
        
        let uri: http::Uri = uri.try_into()
            .change_context_lazy(|| TransportError::Request)?;
        
        let req = http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("date", "")
            .header("host", authority)
            .header("content-type", "application/activity+json")
            .body(reqwest::Body::from(body))
            .change_context_lazy(|| TransportError::Request)?;
        
        let req = req.digest::<Sha256Hasher>().await
            .change_context_lazy(|| TransportError::Digest)?;
        
        let req = req.sign(&*self.signer, &SIGNATURE_PARAMS).await
            .change_context_lazy(|| TransportError::Sign)?;
        
        let req = req.proof(&*self.signer, &SIGNATURE_PARAMS).await
            .change_context_lazy(|| TransportError::Sign)?;
        
        let req = req.map(reqwest::Body::wrap);
        
        self.client.execute(reqwest::Request::try_from(req).unwrap()).await
            .change_context_lazy(|| TransportError::Io)?;
        
        Ok(())
    }
    
    pub(crate) async fn fetch<T>(&self, uri: impl AsRef<str>) -> Result<Truth<T>, Report<InquiryError>>
    where
        T: serde::de::DeserializeOwned
    {
        let res = self.client.get(uri.as_ref()).send().await
            .change_context_lazy(|| InquiryError::NotResponded)?;
        
        let res: http::Response<reqwest::Body> = res.into();
        
        let body: T = match res.body().as_bytes() {
            None => return Err(Report::new(InquiryError::NotResponded)),
            Some(bytes) => {
                serde_json::from_slice(bytes)
                    .change_context_lazy(|| InquiryError::Deserialization)?
            }
        };
        
        if let Err(warn) = Box::pin(self.verify(res)).await {
            return Ok(Truth::False {
                value: body,
                error: warn
            })
        };
        
        Ok(Truth::True(body))
    }
    
    pub(crate) async fn verify<B>(&self, payload: impl Into<ReqOrRes<B>>) -> Result<ReqOrRes<Body>, Report<VerificationError>>
    where
        B: http_body::Body + Send,
        B::Data: Send
    {
        // Deserialize only the publicKey scheme for signature verification.
        // See https://docs.joinmastodon.org/spec/activitypub/#publicKey
        #[derive(Debug, Deserialize, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct PublicKeyScheme {
            public_key: PublicKey
        }
        
        let payload = payload.into();
        let input = SignatureInput::try_from(&payload)
            .change_context_lazy(|| VerificationError)?;
        
        let PublicKeyScheme { public_key } = self.fetch(input.key_id()).await
            .change_context_lazy(|| VerificationError)?
            .ignore();
        
        let verifier = RsaVerifierKey::load(
            input.key_id().to_string(),
            public_key.public_key_pem()
        ).change_context_lazy(|| VerificationError)?;
        
        let payload = match payload {
            ReqOrRes::Request(req) => {
                let req = req.verify_digest::<Sha256Hasher>().await
                    .change_context_lazy(|| VerificationError)?;
                req.verify_sign(&verifier).await
                    .change_context_lazy(|| VerificationError)?;
                ReqOrRes::Request(req)
            },
            ReqOrRes::Response(res) => {
                let res = res.verify_digest::<Sha256Hasher>().await
                    .change_context_lazy(|| VerificationError)?;
                res.verify_sign(&verifier).await
                    .change_context_lazy(|| VerificationError)?;
                ReqOrRes::Response(res)
            }
        };
        
        Ok(payload)
    }
}

#[derive(Debug)]
pub enum ReqOrRes<B> {
    Request(http::Request<B>),
    Response(http::Response<B>)
}

impl<B> ReqOrRes<B> {
    pub fn map<F, C>(self, f: F) -> ReqOrRes<C>
    where
        F: FnOnce(B) -> C
    {
        match self {
            ReqOrRes::Request(req) => {
                let (parts, body) = req.into_parts();
                ReqOrRes::Request(http::Request::from_parts(parts, f(body)))
            },
            ReqOrRes::Response(res) => {
                let (parts, body) = res.into_parts();
                ReqOrRes::Response(http::Response::from_parts(parts, f(body)))
            }
        }
    }
}

impl<B> From<http::Request<B>> for ReqOrRes<B> {
    fn from(value: http::Request<B>) -> Self {
        Self::Request(value)
    }
}

impl<B> From<http::Response<B>> for ReqOrRes<B> {
    fn from(value: http::Response<B>) -> Self {
        Self::Response(value)
    }
}

impl<B> TryFrom<&ReqOrRes<B>> for SignatureInput
where
    B: http_body::Body + Send,
    B::Data: Send
{
    type Error = SignatureInputError;
    
    fn try_from(value: &ReqOrRes<B>) -> Result<Self, Self::Error> {
        match value {
            ReqOrRes::Request(req) => req.try_into(),
            ReqOrRes::Response(res) => res.try_into()
        }
    }
}

#[derive(Debug)]
pub enum Truth<T> {
    True(T),
    False {
        value: T,
        error: Report<VerificationError>
    }
}

impl<T> Truth<T> {
    pub fn ignore(self) -> T {
        match self {
            Truth::True(value) |
            Truth::False { value, .. } => value
        }
    }
}