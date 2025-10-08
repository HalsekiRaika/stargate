use std::ops::Deref;
use std::sync::Arc;
use error_stack::{Report, ResultExt};
use app_cmd::config::DependOnAppConfig;
use app_cmd::interactors::DependOnRelayFollowAcceptInteractor;
use driver::client::http::HttpClient;
use driver::config::Config;
use driver::keyset::RsaVerifierKey;
use driver::middleware::httpsig::{DependOnHttpSignatureVerifier, HttpSignatureVerifierClient};
use driver::remote::{ActorInquiryClient, InboxTransportClient};
use kernel::interface::remotes::{DependOnRemoteActorInquiry, DependOnRemoteInboxTransport};

use crate::error::UnrecoverableError;

pub async fn init(config: Config) -> Result<AppModule, Report<UnrecoverableError>> {
    let http_client = HttpClient::setup(config.clone())
        .change_context(UnrecoverableError)?;
    
    let pub_key = RsaVerifierKey::read_local_file(config.clone())
        .change_context(UnrecoverableError)?;
    
    Ok(AppModule(
        Arc::new(Handler {
            host_name: config.server.host_name,
            host_pubkey: pub_key.as_pem().to_string(),
            http_signature_verifier_client: HttpSignatureVerifierClient::new(http_client.clone()),
            remote_actor_inquiry_client: ActorInquiryClient::new(http_client.clone()),
            inbox_transport_client: InboxTransportClient::new(http_client)
        })
    ))
}

#[derive(Debug)]
pub struct AppModule(Arc<Handler>);

impl Clone for AppModule {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Deref for AppModule {
    type Target = Handler;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Handler {
    host_name: String,
    host_pubkey: String,
    http_signature_verifier_client: HttpSignatureVerifierClient,
    remote_actor_inquiry_client: ActorInquiryClient,
    inbox_transport_client: InboxTransportClient,
}

impl Handler {
    pub fn host_name(&self) -> &str {
        &self.host_name
    }
    
    pub fn host_pubkey(&self) -> &str {
        &self.host_pubkey
    }
}

impl DependOnAppConfig for Handler {
    fn host_name(&self) -> &str {
        &self.host_name
    }
}

impl DependOnHttpSignatureVerifier for Handler {
    type HttpSignatureVerifier = HttpSignatureVerifierClient;
    
    fn http_signature_verifier(&self) -> &Self::HttpSignatureVerifier {
        &self.http_signature_verifier_client
    }
}

impl DependOnRemoteActorInquiry for Handler {
    type RemoteActorInquiry = ActorInquiryClient;
    
    fn remote_actor_inquiry(&self) -> &Self::RemoteActorInquiry {
        &self.remote_actor_inquiry_client
    }
}

impl DependOnRemoteInboxTransport for Handler {
    type RemoteInboxTransport = InboxTransportClient;
    
    fn remote_inbox_transport(&self) -> &Self::RemoteInboxTransport {
        &self.inbox_transport_client
    }
}

impl DependOnRelayFollowAcceptInteractor for Handler {
    type RelayFollowAcceptInteractor = Self;
    fn relay_follow_accept_interactor(&self) -> &Self::RelayFollowAcceptInteractor { self }
}

