mod tcp;
pub mod utils;
use crate::RMI_ID;
use crate::{error::RMIError, remote::RMIResult};
use serde::{Deserialize, Serialize};
pub use tcp::{IpAddr, Ipv4Addr, SocketAddr, TcpClient, TcpListener, TcpStream, send_data, receive_data};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RMIRequest {
    pub object_id: RMI_ID,
    pub method_name: String, //TODO switch to enum
    pub serialized_args: Vec<u8>,
}
impl RMIRequest {
    pub fn new(object_id: RMI_ID, method_handler: String, serialized_args: Vec<u8>) -> RMIRequest {
        RMIRequest {
            object_id,
            method_name: method_handler,
            serialized_args,
        }
    }
}

impl Default for RMIRequest {
    fn default() -> RMIRequest {
        RMIRequest {
            object_id: 42,
            method_name: "test".into(),
            serialized_args: vec![0, 1, 2],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RMIResponse {
    pub result: RMIResult<Vec<u8>>,
}

impl RMIResponse {
    pub fn success(data: Vec<u8>) -> Self {
        RMIResponse { result: Ok(data) }
    }
    pub fn error(msg: String) -> Self {
        RMIResponse {
            result: Err(RMIError::TransportError(msg)),
        }
    }
}

pub trait Transport: Send + Sync {
    fn send<REQ:Serialize + for<'de> Deserialize<'de>, RES: Serialize + for<'de>Deserialize<'de>>(&self, req: REQ) -> RMIResult<RES>;
}
