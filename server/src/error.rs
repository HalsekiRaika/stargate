#[derive(Debug, thiserror::Error)]
#[error("An unrecoverable error occurred...")]
pub struct UnrecoverableError;