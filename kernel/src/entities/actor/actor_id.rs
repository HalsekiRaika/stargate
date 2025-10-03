use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use url::Url;
use crate::errors::KernelError;

/// Represents the unique identifier as url for an actor in the ActivityPub protocol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActorId(Url);

impl ActorId {
    pub fn new(id: impl AsRef<str>) -> Result<Self, Report<KernelError>> {
        Ok(Self(Url::parse(id.as_ref()).change_context_lazy(|| KernelError::Parse)?))
    }
    
    pub fn authority(&self) -> &str {
        self.0.authority()
    }
}

impl AsRef<str> for ActorId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<ActorId> for String {
    fn from(value: ActorId) -> Self {
        value.0.into()
    }
}

impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}