use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use serde::Deserialize;

use crate::remote::{RemoteRef,RMIResult};
use crate::TcpTransport;
use crate::transport::{RMIRequest, Transport};
use crate::error::RMIError;

pub trait RemoteTrait: Send + Sync{
    fn run_method<T: for<'de> Deserialize<'de>>(&self, arg: i32) -> RMIResult<T>;
}

pub struct Stub{
    remote: RemoteRef,
}

impl Stub{
    pub fn new(remote: RemoteRef) -> Self{
        Stub {remote}
    }

    pub fn from(remote: RemoteRef) -> Self{
        Stub{remote}
    }
}

impl RemoteTrait for Stub{
    fn run_method<T: for<'de> Deserialize<'de>> (&self, arg: i32) -> RMIResult<T>{
        // should marshal args into binary format | serde_cbor cause bincode is deprecated. TODO ask Rob or Badia for alternative
        // construct an RMI request struct | guess this should also be serialized but maybe in transport layer
        // use transport to send and get response
        // unmasrhal result & return
        let serialized_args = serde_cbor::to_vec(&(arg))
                            .map_err(|e| RMIError::SerializationError(e.to_string()))?;

        let req = RMIRequest{
            object_id: self.remote.id,
            method_name: "method_name".into(),
            serialized_args,
        };
        let server_addr = self.remote.addr;
        let transport =TcpTransport::new(server_addr) ;
        let response = transport.send(req)?;

        let bytes:Vec<u8>= response.result?;

        let result: T = serde_cbor::from_slice(&bytes)
                                .map_err(|e| RMIError::SerializationError(e.to_string()))?;
        Ok(result)
    }
}