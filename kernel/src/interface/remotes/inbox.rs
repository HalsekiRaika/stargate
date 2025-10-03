use crate::entities::activity::Activity;
use crate::interface::error::Delegate;

pub trait RemoteInboxTransport: 'static + Sync + Send {
    fn transport(&self, to: &str, activity: &Activity) -> impl Future<Output = Result<(), Delegate>> + Send;
}

pub trait DependOnRemoteInboxTransport: 'static + Sync + Send {
    type RemoteInboxTransport: RemoteInboxTransport;
    fn remote_inbox_transport(&self) -> &Self::RemoteInboxTransport;
}
