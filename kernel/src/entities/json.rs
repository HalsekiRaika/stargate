use crate::entities::activity::ActivityType;
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};

/// A wrapper that holds both the original JSON value and the deserialized activity object.
///
/// This is ideal for data that uses the original request, such as `Accept` activity.
#[derive(Debug, Clone)]
pub struct ActivityJson<T> {
    pub original: serde_json::Value,
    pub activity: T,
}

impl<'de, T> Deserialize<'de> for ActivityJson<T>
where
    T: DeserializeOwned + ActivityType,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let original = serde_json::Value::deserialize(deserializer)?;

        original
            .get("type")
            .and_then(|v| v.as_str())
            .filter(|&t| t == T::OBJECT_TYPE)
            .ok_or_else(|| Error::custom(format!("Expected type {}", T::OBJECT_TYPE)))?;

        let activity = T::deserialize(&original).map_err(Error::custom)?;

        Ok(Self { original, activity })
    }
}

mod v2 {
    use error_stack::{ResultExt, Report};
    use crate::entities::activity::types::{Accept, Follow};
    use serde::de::{DeserializeOwned, Error};
    use serde::{Deserialize, Deserializer, Serialize};
    use crate::entities::activity::ActivityType;
    use crate::errors::KernelError;
    
    #[derive(Debug, Clone, Deserialize)]
    #[serde(tag = "type")]
    pub enum Activity {
        Follow(InheritJson<Follow>),
        Accept(Accept),
        #[serde(other)]
        Unknown,
    }
    
    #[derive(Debug, Clone)]
    pub struct InheritJson<T: ActivityType> {
        activity: T,
        original: Object
    }
    
    impl<T: ActivityType> InheritJson<T> {
        pub fn new(activity: T, original: Object) -> Self {
            Self { activity, original }
        }
    }
    
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct Object(serde_json::Value);
    
    impl Object {
        pub(crate) fn new(json: serde_json::Value) -> Self {
            Self(json)
        }
        
        pub fn cast<T>(self) -> Result<T, Report<KernelError>>
        where
            T: DeserializeOwned + ActivityType
        {
            serde_json::from_value::<T>(self.0)
                .change_context_lazy(|| KernelError::Deserialize)
        }
    }
    
    impl<'de, T> Deserialize<'de> for InheritJson<T>
    where
        T: DeserializeOwned + ActivityType
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
        {
            let original = serde_json::Value::deserialize(deserializer)?;
            let activity = T::deserialize(&original).map_err(Error::custom)?;
            
            Ok(Self { activity, original: Object::new(original) })
        }
    }
}
