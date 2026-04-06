mod tcp;
pub mod utils;
use crate::RMI_ID;
use crate::remote::RMIResult;
use crate::stub::{Deserialize, Serialize};
pub use tcp::{IpAddr, SocketAddr, TcpClient, TcpListener, TcpStream, receive_data, send_data};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct RMIRequest {
    pub object_id: RMI_ID,
    pub method_name: String, //TODO switch to enum
    pub serialized_args: Vec<u8>,
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

pub trait Transport: Send + Sync {
    fn send<
        REQ: Serialize + for<'de> Deserialize<'de>,
        RES: Serialize + for<'de> Deserialize<'de>,
    >(
        &self,
        req: REQ,
    ) -> RMIResult<RES>;
}
