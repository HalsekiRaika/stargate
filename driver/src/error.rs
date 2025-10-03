#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("An IO error occurred.")]
    Io,
    #[error("Invalid toml format.")]
    InvalidFormat,
}

#[derive(Debug, thiserror::Error)]
pub enum KeyLoadError {
    #[error("An IO error occurred.")]
    Io,
    #[error("Incorrect key format, etc.")]
    IncorrectKey
}

#[derive(Debug, thiserror::Error)]
#[error("client is not properly setup.")]
pub struct SetupError;

#[derive(Debug, thiserror::Error)]
#[error("httpsig verification failed.")]
pub struct VerificationError;

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("payload cannot be serialized.")]
    Serialization,
    #[error("request cannot be built.")]
    Request,
    #[error("payload cannot be digest.")]
    Digest,
    #[error("payload cannot be signed.")]
    Sign,
    #[error("payload cannot be transport with reqwest.")]
    Io,
}

#[derive(Debug, thiserror::Error)]
pub enum InquiryError {
    #[error("response cannot be deserialized.")]
    Deserialization,
    #[error("response digest is invalid.")]
    InvalidDigest,
    #[error("response verifier key is invalid.")]
    InvalidVerifierKey,
    #[error("response signature is invalid.")]
    InvalidSignature,
    #[error("response does not exist.")]
    NotResponded,
}