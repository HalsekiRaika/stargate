#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("An error occurred in driver layer.")]
    Driver,
    #[error("An error occurred in kernel layer.")]
    Kernel
}