use std::io::{Write,Read};
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use std::net::TcpStream;
use std::thread::spawn;

use crate::transport::{RMIRequest, RMIResponse, Transport};
use crate::error::RMIError;
use crate::registry::{Registry};
use crate::skeleton::{Skeleton};
use crate::remote::{RMIResult};


pub struct TcpTransport{
    server_addr: SocketAddr,
}

impl TcpTransport{
    pub fn new(server_addr: SocketAddr) -> Self{
        TcpTransport { server_addr }
    }
}
impl Transport for TcpTransport{
    fn send(&self, req: RMIRequest) -> RMIResult<RMIResponse>{
        // connect to server
        // serialize request
        // tcpstream first send byte length then bytes
        // get response
        let mut stream = TcpStream::connect(self.server_addr)
            .map_err(|e| RMIError::TransportError(e.to_string()))?;

        let request_serialized = serde_cbor::to_vec(&req)
            .map_err(|e| RMIError::SerializationError(e.to_string()))?;

        let len = request_serialized.len() as u32;
        let _ = stream.write_all(&len.to_be_bytes()).map_err(|e| RMIError::TransportError(e.to_string()))?;
        let _ = stream.write_all(&request_serialized).map_err(|e| RMIError::TransportError(e.to_string()))?;
        let _ = stream.flush().map_err(|e| RMIError::TransportError(e.to_string()))?;

        // how many bytes are we getting back?
        let mut len_response_bytes = [0u8;4];
        let _ = stream.read_exact(&mut len_response_bytes);
        let response_len = u32::from_be_bytes(len_response_bytes) as usize;

        let mut response_bytes = vec![0u8;response_len];       
        let _ = stream.read_exact(&mut response_bytes);

        let response: RMIResponse = serde_cbor::from_slice(&response_bytes)
                                .map_err(|e| RMIError::SerializationError(e.to_string()))?;
        Ok(response)
    }
}


pub struct TcpServer{
    registry: Arc<Registry>,// resource count cause might be used by multiple
    skeleton: Arc<Skeleton>,
}

impl TcpServer{
    pub fn new(registry: Arc<Registry>) -> Self{
        TcpServer {
            registry,
            skeleton: Arc::new(Skeleton::new()),
        }
    }

    pub fn bind(&self, addr: SocketAddr) -> RMIResult<()>{
        let listener = TcpListener::bind(addr)
            .map_err(|e| RMIError::TransportError(e.to_string()))?;
        println!("RMI Server listening on {}", addr);
        loop{
            match listener.accept(){
                Ok((stream, client_addr)) => {
                    println!("New connection from {}", client_addr);
                    let registry = Arc::clone(&self.registry);
                    let skeleton = Arc::clone(&self.skeleton);

                    spawn(move || {
                        if let Err(e) = Self::handle_connection(stream, registry, skeleton){
                            eprintln!("Error handling connection from {}: {}", client_addr, e);
                        }
                    });
                },
                Err(e) => {
                    eprintln!("Error accepting connection: {}",e)
                }
            }
        }
    }

    fn handle_connection(
        mut stream: TcpStream,
        registry: Arc<Registry>,
        skeleton: Arc<Skeleton>,
    ) -> RMIResult<()>{
        let mut len_bytes = [0u8; 4];
        let _ = stream.read_exact(&mut len_bytes);
        let len = u32::from_be_bytes(len_bytes) as usize;

        let request_bytes = vec![0u8; len]; // same thing as when client gets RMIResponse
        let request: RMIRequest = serde_cbor::from_slice(&request_bytes)
                        .map_err(|e| RMIError::SerializationError(e.to_string()))?;

        let object = registry.get(request.object_id)?;
        let response = skeleton.handle_request(request, object.as_ref());

        let response_bytes = serde_cbor::to_vec(&response)
                        .map_err(|e| RMIError::SerializationError(e.to_string()))?;
        let len = response_bytes.len() as u32;

        stream.write_all(&len.to_be_bytes()).map_err(|e| RMIError::TransportError(e.to_string()))?;
        stream.write_all(&response_bytes).map_err(|e| RMIError::TransportError(e.to_string()))?;
        stream.flush().map_err(|e| RMIError::TransportError(e.to_string()))?;//same thing as when client sends RMIRequest

        Ok(())
    }
}