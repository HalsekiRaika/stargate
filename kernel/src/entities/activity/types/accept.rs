use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use crate::entities::activity::{Activity, ActivityId, ActivityType};
use crate::entities::actor::ActorId;
use crate::errors::KernelError;

/// Represents an Accept activity in the ActivityPub protocol.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Accept {
    pub(crate) id: ActivityId,
    pub(crate) actor: ActorId,
    /// The object being accepted, typically a Follow activity.
    pub(crate) object: serde_json::Value,
}

impl Accept {
    pub fn new<O>(actor: ActorId, object: O) -> Result<Self, Report<KernelError>>
    where
        O: Serialize
    {
        Ok(Self {
            id: ActivityId::new(format!("{}/activity/accept", actor)),
            actor,
            object: serde_json::to_value(object)
                .change_context_lazy(|| KernelError::Serialize)?,
        })
    }
    
    pub fn id(&self) -> &ActivityId {
        &self.id
    }
    
    pub fn actor(&self) -> &ActorId {
        &self.actor
    }
    
    pub fn object(&self) -> &serde_json::Value {
        &self.object
    }
}

impl From<Accept> for Activity {
    fn from(value: Accept) -> Self {
        Self::Accept(value)
    }
}

impl ActivityType for Accept {
    const LD_CONTEXT: &'static [&'static str] = &[
        "https://www.w3.org/ns/activitystreams"
    ];
    
    const OBJECT_TYPE: &'static str = "Accept";
}