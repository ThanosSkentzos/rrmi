use std::net::SocketAddr;
use crate::error::RMIError;
use crate::registry::RMI_ID;

use serde::{Deserialize, Serialize};

#[derive(Clone,Serialize,Deserialize)]
pub struct RemoteRef{//should point to RemoteObject on the server side
    pub addr: SocketAddr,   // 127.0.0.1:8080 for example
    pub id: RMI_ID,         // just a num for identity
}

pub trait RemoteObject: Send + Sync{
    fn run(&self, method_name: &str, args: Vec<u8>) -> RMIResult<Vec<u8>>;
}

pub type RMIResult<T> = Result<T, RMIError>;