use std::fmt::Debug;
use std::sync::{Arc, LazyLock};
use std::time::SystemTime;
use error_stack::{Report, ResultExt};
use http::Method;
use http_msgsign_draft::digest::body::Body;
use http_msgsign_draft::digest::Digest;
use http_msgsign_draft::errors::SignatureInputError;
use http_msgsign_draft::sign::{RequestSign, SignatureParams};
use http_msgsign_draft::sign::headers::SignatureInput;
use serde::{Deserialize};
use kernel::entities::activity::Activity;
use kernel::entities::links::types::PublicKey;

use crate::config::Config;
use crate::error::{InquiryError, SetupError, TransportError, VerificationError};
use crate::hasher::Sha256Hasher;
use crate::signature::{RsaSignerKey, RsaVerifierKey};

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    signer: Arc<RsaSignerKey>,
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
        
        for (host, overrides) in config.server.overrides {
            if let Some(cert) = overrides.certificate {
                let cert = reqwest::Certificate::from_pem(
                    std::fs::read(&cert)
                        .change_context_lazy(|| SetupError)
                        .attach_with(|| format!("Cannot read {host} certificate file."))
                        .attach_with(|| format!("{cert} could not be read, or may not exist."))?
                        .as_slice()
                ).change_context_lazy(|| SetupError)?;
                
                tracing::debug!("Add custom root certificate for {host}.");
                client = client.add_root_certificate(cert);
            }
            
            if let Some(resolve) = overrides.resolve {
                tracing::debug!(name: "reqwest::resolve", "{host} resolves to `{resolve:?}`.");
                client = client.resolve(&host, resolve.to_socket_addr(8080));
            }
        }
        
        let client = client.build()
            .change_context_lazy(|| SetupError)?;
        
        let signer = RsaSignerKey::load(
            config.server.host_name,
            "relay.actor".to_string(),
            config.server.keypair.private
        ).change_context_lazy(|| SetupError)?;
        
        Ok(Self {
            client,
            signer: Arc::new(signer),
        })
    }
    
    pub async fn send_activity(&self, uri: impl AsRef<str>, activity: &Activity) -> Result<(), Report<TransportError>> {
        let body = serde_json::to_vec(&activity.clone().into_json_ld())
            .change_context_lazy(|| TransportError::Serialization)?;
        
        let uri = uri.as_ref().parse::<http::Uri>()
            .change_context_lazy(|| TransportError::Request)
            .attach_with(|| format!("`{}` is not a valid URI.", uri.as_ref()))?;
        
        let authority = uri.authority()
            .map(ToString::to_string)
            .unwrap_or(String::new());
        
        let req = http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("date", httpdate::fmt_http_date(SystemTime::now()))
            .header("host", authority)
            .header("content-type", "application/activity+json")
            .body(reqwest::Body::from(body))
            .change_context_lazy(|| TransportError::Request)
            .attach("failed request build.")?;
        
        let req = req.digest::<Sha256Hasher>().await
            .change_context_lazy(|| TransportError::Digest)
            .attach("failed digest.")?;
        
        let req = req.sign(&*self.signer, &SIGNATURE_PARAMS).await
            .change_context_lazy(|| TransportError::Sign)
            .attach("failed sign.")?;
        
        let req = req.proof(&*self.signer, &SIGNATURE_PARAMS).await
            .change_context_lazy(|| TransportError::Sign)
            .attach("failed sign as Authorization")?;
        
        let req = req.map(reqwest::Body::wrap);
        
        self.client.execute(reqwest::Request::try_from(req).unwrap()).await
            .change_context_lazy(|| TransportError::Io)
            .attach("fuck")?;
        
        Ok(())
    }
    
    #[tracing::instrument(skip_all, name = "fetch")]
    pub(crate) async fn fetch<T>(&self, uri: impl AsRef<str>) -> Result<UnverifiedObject<T>, Report<InquiryError>>
    where
        T: serde::de::DeserializeOwned
    {
        let uri = uri.as_ref();
        let res = self.client.get(uri)
            .header("Accept", "application/activity+json")
            .send()
            .await
            .change_context_lazy(|| InquiryError::NotResponded)
            .attach_with(|| format!("Unable to establish connection with `{uri}`."))?;
        
        let response: http::Response<reqwest::Body> = res.into();
        
        let (parts, body) = response.into_parts();
        
        let body = http_body_util::BodyExt::collect(body)
            .await
            .unwrap()
            .to_bytes();
        
        let value: T = serde_json::from_slice(&body)
            .change_context_lazy(|| InquiryError::Deserialization)?;
        
        Ok(UnverifiedObject {
            value,
            response: http::Response::from_parts(parts, reqwest::Body::from(body))
        })
    }
    
    #[tracing::instrument(skip_all, name = "verify")]
    pub(crate) async fn verify<B>(&self, payload: impl Into<ReqOrRes<B>>) -> Result<ReqOrRes<Body>, Report<VerificationError>>
    where
        B: http_body::Body + Send + Debug,
        B::Data: Send
    {
        // Deserialize only the publicKey scheme for signature verification.
        // See https://docs.joinmastodon.org/spec/activitypub/#publicKey
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PublicKeyScheme {
            public_key: PublicKey
        }
        
        let payload = payload.into();
        
        tracing::debug!("\n{payload:#?}");
        
        let input = SignatureInput::try_from(&payload)
            .change_context_lazy(|| VerificationError)
            .attach("`SignatureInput` does not exist.")?;
        
        let PublicKeyScheme { public_key } = self.fetch(input.key_id()).await
            .change_context_lazy(|| VerificationError)
            .attach("PublicKey could not be obtained.")?
            .ignore();
        
        let verifier = RsaVerifierKey::new(input.key_id().to_string(), public_key.public_key_pem())
            .change_context_lazy(|| VerificationError)
            .attach("Cannot load public_key.")?;
        
        let payload = match payload {
            ReqOrRes::Request(req) => {
                let req = req.verify_digest::<Sha256Hasher>().await
                    .change_context_lazy(|| VerificationError)
                    .attach("Digest unverified")?;
                input.verify_request(&req, &verifier)
                    .change_context_lazy(|| VerificationError)
                    .attach("Signature unverified")?;
                ReqOrRes::Request(req)
            },
            ReqOrRes::Response(res) => {
                let res = res.verify_digest::<Sha256Hasher>().await
                    .change_context_lazy(|| VerificationError)
                    .attach("Digest unverified")?;
                input.verify_response(&res,  &verifier)
                    .change_context_lazy(|| VerificationError)
                    .attach("Signature unverified")?;
                ReqOrRes::Response(res)
            }
        };
        
        tracing::debug!("payload verified.");
        
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

pub struct UnverifiedObject<T, B = reqwest::Body>
where
    B: http_body::Body + Send,
    B::Data: Send
{
    value: T,
    response: http::Response<B>
}

impl<T, B> UnverifiedObject<T, B>
where
    B: http_body::Body + Send + Debug,
    B::Data: Send
{
    pub fn ignore(self) -> T {
        self.value
    }
    
    pub async fn verify(self, client: &HttpClient) -> Result<T, Report<VerificationError>> {
        client.verify(self.response).await?;
        Ok(self.value)
    }
}
