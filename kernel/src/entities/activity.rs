mod activity_id;

pub mod types;

pub use self::activity_id::*;

use serde::{Deserialize, Serialize};

use self::types::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Activity {
    Follow(Follow),
    Accept(Accept),
}

pub trait ActivityType {
    const OBJECT_TYPE: &'static str;
}
