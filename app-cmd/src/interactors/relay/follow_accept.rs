use crate::config::{AppConfig, DependOnAppConfig};
use crate::errors::ApplicationError;
use error_stack::{Report, ResultExt};
use kernel::entities::activity::types::Follow;
use kernel::entities::actor::ActorId;
use kernel::entities::json::ActivityJson;
use kernel::interface::remotes::{
    DependOnRemoteActorInquiry,
    DependOnRemoteInboxTransport,
    RemoteActorInquiry,
    RemoteInboxTransport
};

impl<T> RelayFollowAcceptInteractor for T
where
    T
    : DependOnAppConfig
    + DependOnRemoteInboxTransport
    + DependOnRemoteActorInquiry
{}

pub trait DependOnRelayFollowAcceptInteractor: 'static + Sync + Send {
    type RelayFollowAcceptInteractor: RelayFollowAcceptInteractor;
    fn relay_follow_accept_interactor(&self) -> &Self::RelayFollowAcceptInteractor;
}

pub trait RelayFollowAcceptInteractor
where
    Self: Sync + Send + 'static
        + DependOnAppConfig
        + DependOnRemoteInboxTransport
        + DependOnRemoteActorInquiry
{
    fn execute(&self, activity: ActivityJson<Follow>) -> impl Future<Output = Result<(), Report<ApplicationError>>> + Send {
        async move {
            let follow = &activity.activity;
            let actor = self.remote_actor_inquiry()
                .inquire(follow.actor())
                .await
                .change_context_lazy(|| ApplicationError::Driver)?;
            
            let myself = ActorId::new(format!("https://{}/relay.actor", self.app_config().hostname()))
                .change_context_lazy(|| ApplicationError::Kernel)?;
            
            let accept = activity.accept(myself);
            
            self.remote_inbox_transport()
                .transport(actor.inbox_url(), &accept.into())
                .await
                .change_context_lazy(|| ApplicationError::Driver)?;
            
            Ok(())
        }
    }
}
