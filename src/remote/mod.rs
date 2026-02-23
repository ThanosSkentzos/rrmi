pub mod registry;
pub use registry::{RMI_ID,Registry};

use crate::error::RMIError;
use std::net::{IpAddr, SocketAddr};
use serde::{Deserialize, Serialize};

#[derive(Clone,Serialize,Deserialize)]
pub struct RemoteRef{//should point to RemoteObject on the server side
    pub addr: SocketAddr,   // 127.0.0.1:8080 for example
    pub id: RMI_ID,         // just a num for identity
}
impl RemoteRef{
    pub fn new(addr:SocketAddr,id:RMI_ID) -> Self{
        RemoteRef {addr,id}
    }
    pub fn example() -> Self{
        let addr = SocketAddr::new(IpAddr::from([127,0,0,1]), 1099);
        RemoteRef { addr, id: 1}
    }
}

pub trait RemoteObject: Send + Sync{
    fn run(&self, method_name: &str, args: Vec<u8>) -> RMIResult<Vec<u8>>;
}

pub type RMIResult<T> = Result<T, RMIError>;