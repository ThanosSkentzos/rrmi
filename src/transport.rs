use serde::{Serialize,Deserialize};
use crate::{error::RMIError, registry::RMI_ID, remote::RMIResult};

#[derive(Serialize,Deserialize,Debug,Clone,PartialEq)]
pub struct RMIRequest{
    pub object_id: RMI_ID,
    pub method_name: String,
    pub serialized_args: Vec<u8>,
}
impl RMIRequest{
    pub fn new(object_id:RMI_ID,method_handler:String,serialized_args:Vec<u8>)->RMIRequest{
        RMIRequest{object_id,method_name: method_handler,serialized_args}
    }
    
    pub fn example()->RMIRequest{
        RMIRequest{object_id:42,method_name: "test".into(),serialized_args:vec![0,1,2]}
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