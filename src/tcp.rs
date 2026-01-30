use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use async_trait::async_trait;

use crate::transport::{RMIRequest, RMIResponse, Transport};
use crate::error::{RMIResult,RMIError};


pub struct TcpTransport{
    server_addr: SocketAddr,
}

impl TcpTransport{
    pub fn new(server_addr: SocketAddr) -> Self{
        TcpTransport { server_addr }
    }
}
#[async_trait]
impl Transport for TcpTransport{
    async fn send(&self, req: RMIRequest) -> RMIResult<RMIResponse>{
        // connect to server
        // serialize request
        // tcpstream first send byte length then bytes
        // get response
        let mut stream = TcpStream::connect(self.server_addr).await?;

        let request_serialized = serde_cbor::to_vec(&req)
            .map_err(|e| RMIError::SerializationError(e))?;

        let len = request_serialized.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&request_serialized).await?;//should map to transport error?
        stream.flush().await?;

        // how many bytes are we getting back?
        let mut len_response_bytes = [0u8;4];
        stream.read_exact(&mut len_response_bytes).await?;
        let response_len = u32::from_be_bytes(len_response_bytes) as usize;

        let mut response_bytes = vec![0u8;response_len];       
        stream.read_exact(&mut response_bytes).await?;

        let response: RMIResponse = serde_cbor::from_slice(&response_bytes)
                                .map_err(|e| RMIError::SerializationError(e))?;
        Ok(response)
    }
}