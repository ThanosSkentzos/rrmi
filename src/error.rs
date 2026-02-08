use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error,Debug,Clone,Serialize,Deserialize)]
pub enum RMIError {

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Bad arguments for method: {0}")]
    BadArguments(String),

    #[error("Object not found with id: {0}")]
    ObjectNotFound(u64),

    #[error("IO error: {0}")]
    IoError(String),
}
