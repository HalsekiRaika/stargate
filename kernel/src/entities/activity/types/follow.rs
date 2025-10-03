use serde::{Deserialize, Serialize};
use crate::entities::activity::{Activity, ActivityId, ActivityType};
use crate::entities::activity::types::Accept;
use crate::entities::actor::ActorId;
use crate::entities::json::ActivityJson;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Follow {
    id: ActivityId,
    actor: ActorId,
    object: serde_json::Value
}

impl Follow {
    pub fn new(actor: ActorId, object: serde_json::Value) -> Self {
        Self { 
            id: ActivityId::new(format!("{}/follow", actor)), 
            actor, 
            object
        }
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

impl From<Follow> for Activity {
    fn from(value: Follow) -> Self {
        Self::Follow(value)
    }
}

impl ActivityJson<Follow> {
    pub fn accept(self, actor: ActorId) -> Accept {
        Accept {
            id: ActivityId::new(format!("{}/activity/accept", actor)),
            actor,
            object: self.original,
        }
    }
}

impl ActivityType for Follow {
    const OBJECT_TYPE: &'static str = "Follow";
}