use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::registry::RMI_ID;

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
    ObjectNotFound(RMI_ID),

    #[error("Object not found with name: {0}")]
    NameNotFound(String),

    #[error("Empty Registry")]
    EmptyRegistry(),

    #[error("IO error: {0}")]
    IoError(String),
}
