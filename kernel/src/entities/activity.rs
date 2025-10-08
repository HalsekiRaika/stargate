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

impl Activity {
    pub fn into_json_ld(self) -> serde_json::Value {
        let ld_context = match self {
            Activity::Follow(_) => Follow::LD_CONTEXT,
            Activity::Accept(_) => Accept::LD_CONTEXT,
        };
        
        let object = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        
        let mut ld_object = serde_json::Map::new();
        ld_object.insert("@context".to_string(), serde_json::Value::Array(
            ld_context.iter().map(|s| serde_json::Value::String(s.to_string())).collect()
        ));
        
        if let serde_json::Value::Object(obj) = object {
            ld_object.extend(obj);
        }
        
        serde_json::Value::Object(ld_object)
    }
}

pub trait ActivityType {
    const LD_CONTEXT: &'static [&'static str];
    const OBJECT_TYPE: &'static str;
}

#[cfg(test)]
mod test {
    use crate::entities::actor::ActorId;
    use super::*;
    #[test]
    fn ser_ld() {
        let activity = Activity::Follow(Follow::new(
            ActorId::new("https://example.com/actor/alice").unwrap(),
            serde_json::json!("https://example.com/activities/follow1")
        ));
        let json_ld = activity.into_json_ld();
        println!("{}", serde_json::to_string_pretty(&json_ld).unwrap());
        
    }
}