use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};
use crate::entities::activity::ActivityType;


/// A wrapper that holds both the original JSON value and the deserialized activity object.
///
/// This is ideal for data that uses the original request, such as `Accept` activity.
#[derive(Debug, Clone)]
pub struct ActivityJson<T> {
    pub original: serde_json::Value,
    pub activity: T
}

impl<'de, T> Deserialize<'de> for ActivityJson<T>
where
    T: DeserializeOwned + ActivityType
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let original = serde_json::Value::deserialize(deserializer)?;
        
        original.get("type")
            .and_then(|v| v.as_str())
            .filter(|&t| t == T::OBJECT_TYPE)
            .ok_or_else(|| Error::custom(format!("Expected type {}", T::OBJECT_TYPE)))?;
        
        let activity = T::deserialize(&original).map_err(Error::custom)?;
        
        Ok(Self { original, activity })
    }
}

#[cfg(test)]
mod test {
    use crate::entities::activity::types::Follow;
    use crate::entities::json::ActivityJson;
    
    #[test]
    fn original_activity_json_held_deserialize() {
        let json: ActivityJson<Follow> = serde_json::from_str(r#"
        {
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": "https://example.com/activities/12345",
            "type": "Follow",
            "actor": "https://example.com/users/alice",
            "object": "https://example.com/users/bob"
        }
        "#).unwrap();
        
        println!("{:#?}", json);
    }
}
