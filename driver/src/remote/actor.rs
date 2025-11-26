use error_stack::Report;
use kernel::entities::actor::{Actor, ActorId};
use kernel::interface::error::Delegate;
use kernel::interface::remotes::RemoteActorInquiry;

use crate::client::http::HttpClient;
use crate::error::InquiryError;

#[derive(Debug, Clone)]
pub struct ActorInquiryClient {
    client: HttpClient
}

impl ActorInquiryClient {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

impl RemoteActorInquiry for ActorInquiryClient {
    async fn inquire(&self, actor: &ActorId) -> Result<Actor, Delegate> {
        let actor = ActorInquiryClientInternal::inquire_actor(actor, &self.client).await?;
        Ok(actor)
    }
}

pub(crate) struct ActorInquiryClientInternal;

impl ActorInquiryClientInternal {
    pub async fn inquire_actor(actor: &ActorId, client: &HttpClient) -> Result<Actor, Report<InquiryError>> {
        Ok(client.fetch::<Actor>(actor).await?.ignore())
    }
}
