use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ActivityId(String);

impl ActivityId {
    pub fn new(id: impl Into<String>) -> Self {
        ActivityId(id.into())
    }
}

impl AsRef<str> for ActivityId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<ActivityId> for String {
    fn from(id: ActivityId) -> Self {
        id.0
    }
}

impl std::fmt::Display for ActivityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}