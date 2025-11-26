#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("Parsing error")]
    Parse,
    #[error("")]
    Serialize,
    #[error("")]
    Deserialize,
}