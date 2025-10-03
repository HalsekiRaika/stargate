use crate::entities::actor::{Actor, ActorId};
use crate::interface::error::Delegate;

pub trait RemoteActorInquiry: 'static + Sync + Send {
    fn inquire(&self, actor: &ActorId) -> impl Future<Output = Result<Actor, Delegate>> + Send;
}

pub trait DependOnRemoteActorInquiry: 'static + Sync + Send {
    type RemoteActorInquiry: RemoteActorInquiry;
    fn remote_actor_inquiry(&self) -> &Self::RemoteActorInquiry;
}
