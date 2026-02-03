use crate::remote::RemoteObject;
use crate::transport::{RMIRequest, RMIResponse};


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