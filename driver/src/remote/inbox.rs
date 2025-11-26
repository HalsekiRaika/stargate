use error_stack::Report;
use kernel::entities::activity::Activity;
use kernel::interface::error::Delegate;
use kernel::interface::remotes::RemoteInboxTransport;
use crate::client::http::HttpClient;
use crate::error::TransportError;

#[derive(Debug, Clone)]
pub struct InboxTransportClient {
    client: HttpClient
}

impl InboxTransportClient {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

impl RemoteInboxTransport for  InboxTransportClient {
    #[tracing::instrument(skip_all, name = "remote_transport")]
    async fn transport(&self, to: &str, activity: &Activity) -> Result<(), Delegate> {
        InboxTransportClientInternal::transport(to, activity, &self.client).await?;
        Ok(())
    }
}

pub(crate) struct InboxTransportClientInternal;

impl InboxTransportClientInternal {
    pub async fn transport(to: &str, activity: &Activity, client: &HttpClient) -> Result<(), Report<TransportError>> {
        client.send_activity(to, activity).await?;
        Ok(())
    }
}
