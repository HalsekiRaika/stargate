
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ActorType {
    Application,
    Group,
    Organization,
    Person,
    Service,
}
