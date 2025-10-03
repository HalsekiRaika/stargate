use error_stack::{Report, ResultExt};
use http_msgsign_draft::digest::body::Body;
use http_msgsign_draft::sign::headers::SignatureInput;

use crate::client::http::{HttpClient, ReqOrRes};
use crate::error::VerificationError;

pub trait HttpSignatureVerifier: 'static + Sync + Send {
    fn verify<B>(&self, request: http::Request<B>) -> impl Future<Output=Result<http::Request<Body>, Report<VerificationError>>> + Send
    where
        B: http_body::Body + Send + Sync,
        B::Data: Send;
}

pub trait DependOnHttpSignatureVerifier {
    type HttpSignatureVerifier: HttpSignatureVerifier;
    fn http_signature_verifier(&self) -> &Self::HttpSignatureVerifier;
}

#[derive(Debug, Clone)]
pub struct HttpSignatureVerifierClient {
    client: HttpClient
}

impl HttpSignatureVerifierClient {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}


impl HttpSignatureVerifier for HttpSignatureVerifierClient {
    async fn verify<B>(&self, request: http::Request<B>) -> Result<http::Request<Body>, Report<VerificationError>>
    where
        B: http_body::Body + Send,
        B::Data: Send
    {
        let ReqOrRes::Request(request) = self.client.verify(request).await? else {
            unreachable!("On the contrary, I can't even begin to imagine how to get here...")
        };
        Ok(request)
    }
}

