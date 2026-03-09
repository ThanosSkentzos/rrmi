use std::io::{Read, Write};
pub use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::RMIError;
use crate::remote::{RMIResult, Registry};
use crate::transport::{RMIRequest, RMIResponse, Transport};

pub fn send_data(data_serial: Vec<u8>, stream:&mut TcpStream) -> RMIResult<()> {
    let len = data_serial.len() as u32;
    eprintln!("tcp sending {len} bytes...");
    let _ = stream
        .write_all(&len.to_be_bytes())
        .map_err(|e| RMIError::TransportError(e.to_string()))?;
    let _ = stream
        .write_all(&data_serial)
        .map_err(|e| RMIError::TransportError(e.to_string()))?;
    let _ = stream.flush()
        .map_err(|e| RMIError::TransportError(e.to_string()))?;
    Ok(())
}

pub fn receive_data(stream:&mut TcpStream) -> Vec<u8>{
    let mut len_bytes = [0u8; 4];
    let _ = stream.read_exact(&mut len_bytes);
    let response_len = u32::from_be_bytes(len_bytes) as usize;

    eprintln!("tcp reading response {response_len:?} bytes...");
    let mut bytes = vec![0u8; response_len];
    let _ = stream.read_exact(&mut bytes);
    bytes
}

pub struct TcpClient {
    server_addr: SocketAddr,
}

impl TcpClient {
    pub fn new(server_addr: SocketAddr) -> Self {
        TcpClient { server_addr }
    }
}
impl Transport for TcpClient {
    fn send<
        REQ: Serialize + for<'de> Deserialize<'de>,
        RES: Serialize + for<'de> Deserialize<'de>,
    >(
        &self,
        req: REQ,
    ) -> RMIResult<RES> {
        // eprintln!("marshaling");
        let request_serialized = serde_cbor::to_vec(&req).map_err(|e| RMIError::SerializationError(e.to_string()))?;
        // eprintln!("send_data");
        let mut stream = TcpStream::connect(self.server_addr).unwrap();
        send_data(request_serialized, &mut stream)?;// return error to not block
        // eprintln!("receive_data");
        let response_bytes = receive_data(&mut stream);
        // eprintln!("unmarshaling");
        let response: RES = serde_cbor::from_slice(&response_bytes)
            .map_err(|e| RMIError::DeserializationError(e.to_string()))
            .expect("should deserialize");
        Ok(response)
    }
}

pub struct TcpServer {
    registry: Arc<Registry>, // resource count cause might be used by multiple
}

impl TcpServer {
    pub fn new(registry: Arc<Registry>) -> Self {
        TcpServer { registry }
    }

    pub fn bind(&self, addr: SocketAddr) -> RMIResult<()> {
        let listener =
            TcpListener::bind(addr).map_err(|e| RMIError::TransportError(e.to_string()))?;
        eprintln!("RMI Server listening on {}", addr);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client_addr = stream
                        .peer_addr()
                        .unwrap_or_else(|_| "unknown".parse().unwrap());
                    eprintln!("New connection from {}", client_addr);

                    if let Err(e) = &self.handle_connection(stream) {
                        eprintln!("Error handling connection from {}: {}", client_addr, e);
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e)
                }
            }
        }
        Ok(())
    }

    fn handle_connection(&self, mut stream: TcpStream) -> RMIResult<()> {
        let request_bytes = receive_data(&mut stream);
        let request: RMIRequest = serde_cbor::from_slice(&request_bytes)
            .map_err(|e| RMIError::DeserializationError(e.to_string()))?;
        eprintln!(
            "Received request for object_id= {}, method= {}",
            request.object_id, request.method_name
        );

        let object = self.registry.get(&request.object_id)?;
        let response: RMIResult<()> = todo!();
        // let response = self.skeleton.handle_request(request, object.as_ref());

        let response_bytes = serde_cbor::to_vec(&response)
            .map_err(|e| RMIError::SerializationError(e.to_string()))?;
        let len = response_bytes.len() as u32;

        send_data(response_bytes, &mut stream)
    }
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::thread;

    use super::*;
    use crate::utils::{get_local_addr, get_server_addr};

    #[test]
    fn liacs_ips() {
        let hostname = "0.0.0.0";
        get_server_addr(hostname, 1099);
        let hostname = "localhost";
        get_server_addr(hostname, 1099);
        let hostname = "0065074.student.liacs.nl";
        get_server_addr(hostname, 1099);
        let hostname = "0065073.student.liacs.nl";
        get_server_addr(hostname, 1099);
    }

    static LOCAL_GET_SEND:u16 = 10999;
    #[test]
    fn local_get() {
        let num: i32 = 1234567890;
        eprintln!("data: {:?}", num);
        let addr = get_local_addr(LOCAL_GET_SEND);
        // let mut stream = TcpStream::connect(addr).unwrap();
        let listener  = TcpListener::bind(addr).expect("should be free");
        let (mut stream,_) = listener.accept().expect("should send");
        let bytes = receive_data(&mut stream);
        let num_recv:i32 = serde_cbor::from_slice(&bytes).expect("i32");
        assert_eq!(num_recv, num);

        let req = RMIRequest::default();
        let bytes =  receive_data(&mut stream);
        let req_recv: RMIRequest = serde_cbor::from_slice(&bytes).expect("RMIRequest");
        assert_eq!(req_recv, req);
    }

    #[test]
    fn local_send() {
        let addr = get_local_addr(LOCAL_GET_SEND);
        let mut stream = TcpStream::connect(addr).unwrap();
        let int: i32 = 1234567890;
        let int_bytes = serde_cbor::to_vec(&int).expect("int is serializable");
        eprintln!("data: {:?}", int);
        eprintln!("serialized: {:?}", int_bytes);
        thread::sleep(time::Duration::from_millis(1000)); //at first was failing randomly, probably race condition with server thread

        send_data(int_bytes.clone(), &mut stream);

        let request = RMIRequest::default();
        let request_bytes = serde_cbor::to_vec(&request).expect("RMIRequest is serializable");
        thread::sleep(time::Duration::from_millis(300)); //at first was failing randomly, probably race condition with server thread
        send_data(request_bytes, &mut stream);
    }

    #[test]
    fn remote_send_int() {
        let addr = get_server_addr("0065074.student.liacs.nl", 10998);
        let mut stream = TcpStream::connect(addr).unwrap();
        let num: i32 = 1234567890;
        let num_bytes = serde_cbor::to_vec(&num).expect("int is serializable");
        eprintln!("data: {:?}", num);
        eprintln!("serialized: {:?}", num_bytes);

        thread::sleep(time::Duration::from_millis(100)); //at first was failing randomly, probably race condition with server thread
        send_data(num_bytes.clone(),&mut stream);
    }

    #[test]
    fn remote_get_int() {
        let addr = get_server_addr("0065074.student.liacs.nl", 10998);
        let mut stream = TcpStream::connect(addr).unwrap();
        let num: i32 = 1234567890;
        let num_serial = serde_cbor::to_vec(&num).expect("int is serializable");
        eprintln!("data: {:?}", num);
        eprintln!("serialized: {:?}", num_serial);

        let bytes = receive_data(&mut stream);
        let num_recv:i32 = serde_cbor::from_slice(&bytes).expect("i32");
        assert_eq!(num, num_recv);
    }

    #[test]
    fn remote_send_request() {
        let addr = get_server_addr("0065074.student.liacs.nl", 10999);
        let mut stream = TcpStream::connect(addr).unwrap();
        let data = RMIRequest::default();
        let data_serial = serde_cbor::to_vec(&data).expect("RMIRequest is serializable");

        thread::sleep(time::Duration::from_millis(10));
        send_data(data_serial,&mut stream);
    }

    #[test]
    fn remote_get_request() {
        let addr = get_server_addr("0065074.student.liacs.nl", 10999);
        let mut stream = TcpStream::connect(addr).unwrap();
        let req = RMIRequest::default();
        let bytes =  receive_data(&mut stream);
        let req_recv: RMIRequest = serde_cbor::from_slice(&bytes).expect("RMIRequest");
        assert_eq!(req, req_recv)
    }
}
