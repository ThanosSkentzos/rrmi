use serde::{Serialize,Deserialize};
use async_trait::async_trait;
use crate::remote::RMIResult;

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct RMIRequest{
    pub object_id: u64,
    pub method_handler: String,
    pub serialized_args: Vec<u8>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct RMIResponse{
    pub result: Result<Vec<u8>, String>
}

impl RMIResponse{
    pub fn success(data: Vec<u8>) -> Self{
        RMIResponse {
            result: Ok(data),
        }
    }
    pub fn error(msg: String) -> Self{
        RMIResponse { 
            result: Err(msg),
        }
    }
}

#[async_trait]// need this for Send trait, otherwise cannot use dyn Transport
pub trait Transport: Send + Sync{
    async fn send(&self, req: RMIRequest) -> RMIResult<RMIResponse>;
}