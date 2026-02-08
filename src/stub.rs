use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::remote::{RemoteRef,RMIResult};
use crate::tcp::TcpTransport;
use crate::transport::{RMIRequest, Transport};
use crate::error::RMIError;

type ReturnType = i32;

pub trait RemoteTrait: Send + Sync{
    fn run_method(&self, arg: i32) -> RMIResult<ReturnType>;
}

pub struct Stub{
    remote: RemoteRef,
}

impl Stub{
    pub fn new(remote: RemoteRef) -> Self{
        Stub {remote}
    }
}

impl RemoteTrait for Stub{
    fn run_method(&self, arg: i32) -> RMIResult<ReturnType>{
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
        let ip = Ipv4Addr::new(127, 0, 0, 1);//TODO local registry should give ip from remote object
        let ip: IpAddr = IpAddr::V4(ip);
        let port = 9999;
        let server_addr = SocketAddr::new(ip, port);
        let transport =TcpTransport::new(server_addr) ;
        let response = transport.send(req)?;

        let bytes:Vec<u8>= response.result?;
        // map_err(|e| RMIError::ServerError(e.to_string()));

        let result: ReturnType = serde_cbor::from_slice(&bytes)
                                .map_err(|e| RMIError::SerializationError(e.to_string()))?;
        Ok(result)
    }
}