use thiserror::Error;

#[derive(Error,Debug)]
pub enum RMIError {

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_cbor::Error),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Transport error: {0}")]
    TransportError(String),
}

pub type RMIResult<T> = Result<T, RMIError>;