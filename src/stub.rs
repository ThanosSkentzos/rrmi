use std::sync::Arc;
use crate::remote::RemoteRef;
use crate::transport::{RMIRequest, Transport};
use crate::error::{RMIError,RMIResult};
use async_trait::async_trait;

type ReturnType = i32;

#[async_trait]
pub trait RemoteTrait: Send + Sync{
    async fn method_name(&self, arg: i32) -> RMIResult<ReturnType>;
}

pub struct Stub{
    remote: RemoteRef,
    transport: Arc<dyn Transport>
}

impl Stub{
    pub fn new(remote: RemoteRef, transport: Arc<dyn Transport>) -> Self{
        Stub { remote, transport }
    }
}

#[async_trait]
impl RemoteTrait for Stub{
    async fn method_name(&self, arg: i32) -> RMIResult<ReturnType>{
        // should marshal args into binary format | serde_cbor cause bincode is deprecated. TODO ask Rob or Badia for alternative
        // construct an RMI request struct | guess this should also be serialized but maybe in transport layer
        // use transport to send and get response
        // unmasrhal result & return
        let serialized_args = serde_cbor::to_vec(&(arg))
                            .map_err(|e| RMIError::SerializationError(e))?;

        let req = RMIRequest{
            object_id: self.remote.id,
            method_handler: "method_name".into(),
            serialized_args,
        };
        
        let response = self.transport.send(req).await?;// TODO handle error -> what error here? transport or server? actually let send return whichever!!!
        let bytes = response.result
                            .map_err(|e| RMIError::ServerError(e))?;

        let result: ReturnType = serde_cbor::from_slice(&bytes)
                                .map_err(|e| RMIError::SerializationError(e))?;
        Ok(result)
    }
}