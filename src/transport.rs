use serde::{Serialize,Deserialize};
use crate::{error::RMIError, remote::RMIResult};

#[derive(Serialize,Deserialize,Debug,Clone,PartialEq)]
pub struct RMIRequest{
    pub object_id: u64,
    pub method_handler: String,
    pub serialized_args: Vec<u8>,
}
impl RMIRequest{
    pub fn new(object_id:u64,method_handler:String,serialized_args:Vec<u8>)->RMIRequest{
        RMIRequest{object_id,method_handler,serialized_args}
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct RMIResponse{
    pub result: RMIResult<Vec<u8>>,
}

impl RMIResponse{
    pub fn success(data: Vec<u8>) -> Self{
        RMIResponse {
            result: Ok(data),
        }
    }
    pub fn error(msg: String) -> Self{
        RMIResponse { 
            result: Err(RMIError::TransportError(msg)),
        }
    }
}

pub trait Transport: Send + Sync{
    fn send(&self, req: RMIRequest) -> RMIResult<RMIResponse>;
}