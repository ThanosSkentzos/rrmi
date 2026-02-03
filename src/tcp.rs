use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use async_trait::async_trait;

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

    pub async fn bind(&self, addr: SocketAddr) -> RMIResult<()>{
        let listener = TcpListener::bind(addr).await?;
        println!("RMI Server listening on {}", addr);
        loop{
            match listener.accept().await{
                Ok((stream, client_addr)) => {
                    println!("New connection from {}", client_addr);
                    let registry = Arc::clone(&self.registry);
                    let skeleton = Arc::clone(&self.skeleton);

                    tokio::spawn(async move{
                        if let Err(e) = Self::handle_connection(stream, registry, skeleton).await{
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

    async fn handle_connection(
        mut stream: TcpStream,
        registry: Arc<Registry>,
        skeleton: Arc<Skeleton>,
    ) -> RMIResult<()>{
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut request_bytes = vec![0u8; len]; // same thing as when client gets RMIResponse
        let request: RMIRequest = serde_cbor::from_slice(&request_bytes)
                        .map_err(|e| RMIError::SerializationError(e))?;

        let object = registry.get(request.object_id).await?;
        let response = skeleton.handle_request(request, object.as_ref()).await;

        let response_bytes = serde_cbor::to_vec(&response)
                        .map_err(|e| RMIError::SerializationError(e))?;
        let len = response_bytes.len() as u32;

        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;//same thing as when client sends RMIRequest

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote::{RemoteRef,RemoteObject};
    use crate::stub::{Stub, RemoteTrait};
    use std::time::Duration;
    
    struct TestObject;
    
    #[async_trait]
    impl RemoteObject for TestObject {
        async fn run(&self, method_name: &str, args: Vec<u8>) -> RMIResult<Vec<u8>> {
            match method_name {
                "method_name" => {
                    let arg: i32 = serde_cbor::from_slice(&args)?;
                    let result = arg * 2;
                    Ok(serde_cbor::to_vec(&result)?)
                }
                _ => Err(RMIError::MethodNotFound(method_name.to_string())),
            }
        }
    }
    
    #[tokio::test]
    async fn test_tcp_transport() {
        // Start server
        let registry = Arc::new(Registry::new());
        let server = TcpServer::new(Arc::clone(&registry));
        
        let server_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        
        // Register an object
        let obj = Arc::new(TestObject);
        let object_id = registry.register(obj).await;
        
        // Spawn server task
        let server_task = tokio::spawn(async move {
            server.bind(server_addr).await
        });
        
        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Create client
        let transport = Arc::new(TcpTransport::new(server_addr));
        let remote_ref = RemoteRef {
            addr: server_addr,
            id: object_id,
        };
        let stub = Stub::new(remote_ref, transport);
        
        // Make RMI call
        let result = stub.method_name(21).await.unwrap();
        assert_eq!(result, 42);
        
        // Cleanup
        server_task.abort();
    }
}
