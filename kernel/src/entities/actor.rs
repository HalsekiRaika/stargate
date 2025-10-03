mod actor_id;

pub mod types;

pub use self::actor_id::*;

use serde::{Deserialize, Serialize};

use self::types::*;

use crate::entities::links::types::{Image, PublicKey};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    id: ActorId,
    #[serde(rename = "type")]
    actor_type: ActorType,
    inbox: String,
    outbox: String,
    followers: String,
    following: String,
    featured: Option<String>,
    shared_inbox: Option<String>,
    endpoints: Option<Endpoints>,
    url: String,
    preferred_username: String,
    name: Option<String>,
    summary: Option<String>,
    icon: Option<Image>,
    image: Option<Image>,
    tag: Vec<serde_json::Value>,
    manually_approves_followers: Option<bool>,
    discoverable: Option<bool>,
    public_key: PublicKey,
}

impl Actor {
    pub fn id(&self) -> &ActorId {
        &self.id
    }
    
    pub fn actor_type(&self) -> &ActorType {
        &self.actor_type
    }
    
    pub fn inbox_url(&self) -> &str {
        &self.inbox
    }
    
    pub fn key(&self) -> &PublicKey {
        &self.public_key
    }
}

impl PartialEq for Actor {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Actor {}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    shared_inbox: String,
}
