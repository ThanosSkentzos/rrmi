mod tcp;
pub mod utils;
use crate::RMI_ID;
use crate::remote::RMIResult;
use crate::stub::{Deserialize, Serialize};
pub use tcp::{IpAddr, SocketAddr, TcpClient, TcpListener, TcpStream, receive_data, send_data};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct RMIResponse {
//     pub result: RMIResult<Vec<u8>>,
// }

// impl RMIResponse {
//     pub fn success(data: Vec<u8>) -> Self {
//         RMIResponse { result: Ok(data) }
//     }
//     pub fn error(msg: String) -> Self {
//         RMIResponse {
//             result: Err(RMIError::TransportError(msg)),
//         }
//     }
// }

pub trait Transport: Send + Sync {
    fn send<
        REQ: Serialize + for<'de> Deserialize<'de>,
        RES: Serialize + for<'de> Deserialize<'de>,
    >(
        &self,
        req: REQ,
    ) -> RMIResult<RES>;
}
