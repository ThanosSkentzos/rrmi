use async_trait::async_trait;

use crate::error::{RMIResult};
use crate::transport::{RMIRequest, RMIResponse};

#[async_trait]
pub trait RemoteObject: Send + Sync{
    async fn run(&self, method_name: &str, args: Vec<u8>) -> RMIResult<Vec<u8>>;
}

pub struct Skeleton{
}

impl Skeleton{
    pub fn new() -> Self{
        Skeleton {}
    }

    pub async fn handle_request(
        &self,
        request: RMIRequest,
        object: &dyn RemoteObject,
    ) -> RMIResponse{
        match object.run(&request.method_handler, request.serialized_args).await{
            Ok(result) => RMIResponse::success(result),
            Err(e) => RMIResponse::error(format!("{e}")),
        }
    }
}

//#TODO tests