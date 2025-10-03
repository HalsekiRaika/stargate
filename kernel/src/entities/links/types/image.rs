use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Image {
    url: String,
    sensitive: Option<bool>,
    name: Option<String>,
}
