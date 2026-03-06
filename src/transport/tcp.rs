use std::io::{Write,Read};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;

use crate::transport::{RMIRequest, RMIResponse, Transport};
use crate::remote::{Registry,RMIResult};
use crate::error::RMIError;


pub struct TcpClient{
    server_addr: SocketAddr,
}

impl TcpClient{
    pub fn new(server_addr: SocketAddr) -> Self{
        TcpClient { server_addr }
    }
}
impl Transport for TcpClient{
    fn send(&self, req: RMIRequest) -> RMIResult<RMIResponse>{
        // connect to server
        // serialize request
        // tcpstream first send byte length then bytes
        // get response
        // eprintln!("tcp connecting...");
        let mut stream = TcpStream::connect(self.server_addr)
            .map_err(|e| RMIError::TransportError(e.to_string()))?;

        // eprintln!("tcp serializing...");
        let request_serialized = serde_cbor::to_vec(&req)
            .map_err(|e| RMIError::SerializationError(e.to_string()))?;
        let len = request_serialized.len() as u32;

        eprintln!("tcp sending {len} bytes...");
        let _ = stream.write_all(&len.to_be_bytes()).map_err(|e| RMIError::TransportError(e.to_string()))?;
        let _ = stream.write_all(&request_serialized).map_err(|e| RMIError::TransportError(e.to_string()))?;
        let _ = stream.flush().map_err(|e| RMIError::TransportError(e.to_string()))?;

        // eprintln!("tcp reading response len...");
        // how many bytes are we getting back?
        let mut len_response_bytes = [0u8;4];
        let _ = stream.read_exact(&mut len_response_bytes);
        let response_len = u32::from_be_bytes(len_response_bytes) as usize;

        eprintln!("tcp reading response {response_len:?} bytes...");
        let mut response_bytes = vec![0u8;response_len];       
        let _ = stream.read_exact(&mut response_bytes);

        // eprintln!("tcp deserializing...");
        let response: RMIResponse = serde_cbor::from_slice(&response_bytes)
                                .map_err(|e| RMIError::DeserializationError(e.to_string()))?;
        // eprintln!("tcp deserializing...");
        Ok(response)
    }
}


pub struct TcpServer{
    registry: Arc<Registry>,// resource count cause might be used by multiple
}

impl TcpServer{
    pub fn new(registry: Arc<Registry>) -> Self{
        TcpServer {
            registry,
        }
    }

    pub fn bind(&self, addr: SocketAddr) -> RMIResult<()>{
        let listener = TcpListener::bind(addr)
            .map_err(|e| RMIError::TransportError(e.to_string()))?;
        eprintln!("RMI Server listening on {}", addr);
        for stream in listener.incoming(){
            match stream{
                Ok(stream) => {
                    let client_addr = stream.peer_addr().unwrap_or_else(|_|{"unknown".parse().unwrap()});
                    eprintln!("New connection from {}", client_addr);

                    if let Err(e) = &self.handle_connection(stream){
                        eprintln!("Error handling connection from {}: {}", client_addr, e);
                    }
                },
                Err(e) => {
                    eprintln!("Error accepting connection: {}",e)
                }
            }
        }
        Ok(())
    }

    fn handle_connection(&self,mut stream: TcpStream) -> RMIResult<()>{
        let mut len_bytes = [0u8; 4];
        let _ = stream.read_exact(&mut len_bytes);
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut request_bytes = vec![0u8; len]; // same thing as when client gets RMIResponse
        let _ = stream.read_exact(&mut request_bytes);
        let request: RMIRequest = serde_cbor::from_slice(&request_bytes)
                        .map_err(|e| RMIError::DeserializationError(e.to_string()))?;
        eprintln!("Received request for object_id= {}, method= {}",request.object_id,request.method_name);

        let object = self.registry.get(&request.object_id)?;
        let response:RMIResult<()> = todo!();
        // let response = self.skeleton.handle_request(request, object.as_ref());

        let response_bytes = serde_cbor::to_vec(&response)
                        .map_err(|e| RMIError::SerializationError(e.to_string()))?;
        let len = response_bytes.len() as u32;

        stream.write_all(&len.to_be_bytes()).map_err(|e| RMIError::TransportError(e.to_string()))?;
        stream.write_all(&response_bytes).map_err(|e| RMIError::TransportError(e.to_string()))?;
        stream.flush().map_err(|e| RMIError::TransportError(e.to_string()))?;//same thing as when client sends RMIRequest

        eprintln!("Response sent.");
        Ok(())
    }
}


#[cfg(test)]
mod tests{
    use core::{panic, time};
    use std::{fmt::Debug, net::{IpAddr, Ipv4Addr}, str::FromStr, thread};
    use serde::Deserialize;

    use super::*;

    fn get_local_addr()->SocketAddr{
        let hostname = "localhost";
        let ips: Vec<std::net::IpAddr> = dns_lookup::lookup_host(hostname).unwrap().collect();
        eprintln!("{hostname} ips: {ips:?}");
        let port = 10999;
        SocketAddr::new(ips[0], port)// TODO for now use 1st entry
    }

    fn get_server_addr(hostname:&str)->SocketAddr{
        let ips: Vec<std::net::IpAddr> = dns_lookup::lookup_host(hostname).unwrap().collect();
        eprintln!("{hostname} ips: {ips:?}");
        if ips.len()==0{//fail test if not found
            panic!("unable to resolve hostname: {hostname}")
        }
        let port = 10999;
        let mut ip:IpAddr = ips[0];
        if ips.iter().any(|ip| ip.to_string().contains("127.0")){
            ip = IpAddr::from(Ipv4Addr::from_str("0.0.0.0").expect("0.0.0.0 should pass"));
            eprintln!("{hostname} is this computer so using {ip:?}");
        }
        eprintln!("using {}:{port} for {hostname}",ip);
        SocketAddr::new(ip, port)
    }

    #[test]
    fn liacs_ips(){
        let hostname = "0.0.0.0";
        get_server_addr(hostname);
        let hostname = "localhost";
        get_server_addr(hostname);
        let hostname = "0065074.student.liacs.nl";
        get_server_addr(hostname);
        let hostname = "0065073.student.liacs.nl";
        get_server_addr(hostname);
    }

    fn send_data(data_serial:Vec<u8>,addr:SocketAddr){
        eprintln!("Client sending to {addr}");
        let mut stream = TcpStream::connect(addr.to_string()).expect("client stream should be able to connect");
        let len = data_serial.len() as u32;
        let _ = stream.write_all(&len.to_be_bytes()).expect("should be able to write len");
        let _ = stream.write_all(&data_serial).expect("then send data stream");
        let _ = stream.flush().expect("make sure we send this");
    }

    fn receive_data<T: for<'de> Deserialize<'de> + Debug + PartialEq>(addr:SocketAddr) -> T{
        let listener = TcpListener::bind(addr.to_string()).expect("port should be available");
        eprintln!("Server listening on {addr}");
        let stream = listener.accept();
        match stream {
            Ok((mut stream,_addr))=>{
                let mut len_bytes = [0u8; 4];
                let _ = stream.read_exact(&mut len_bytes);
                let len = u32::from_be_bytes(len_bytes) as usize;
                let mut request_bytes = vec![0u8; len]; // same thing as when client gets RMIResponse
                let _ = stream.read_exact(&mut request_bytes);
                let data_recv: T = serde_cbor::from_slice(&request_bytes).expect("type should be deserializable");
                eprintln!("Server received data {:?}",data_recv);
                data_recv 
            },
            Err(e) => panic!("Error accepting connection: {}",e)
        }
    }

    #[test]
    fn local_send() {
        let addr = get_local_addr();
        let int:i32 = 1234567890;
        let int_bytes = serde_cbor::to_vec(&int).expect("int is serializable");
        eprintln!("data: {:?}",int);
        eprintln!("serialized: {:?}",int_bytes);
        thread::sleep(time::Duration::from_millis(10));//at first was failing randomly, probably race condition with server thread

        send_data(int_bytes.clone(), addr);
        
        let request = RMIRequest::default();
        let request_bytes = serde_cbor::to_vec(&request).expect("RMIRequest is serializable");
        thread::sleep(time::Duration::from_millis(10));//at first was failing randomly, probably race condition with server thread
        send_data(request_bytes, addr);
    }

    #[test]
    fn local_get() {
        let num:i32 = 1234567890;
        eprintln!("data: {:?}",num);
        let addr = get_local_addr();
        let num_recv:i32 = receive_data(addr);
        assert_eq!(num_recv,num);
        
        let req = RMIRequest::default();
        let req_recv:RMIRequest = receive_data(addr);
        assert_eq!(req_recv,req);
    }

    #[test]
    fn remote_send_int() {
        let addr = get_server_addr("0065074.student.liacs.nl");
        let num:i32 = 1234567890;
        let num_bytes = serde_cbor::to_vec(&num).expect("int is serializable");
        eprintln!("data: {:?}",num);
        eprintln!("serialized: {:?}",num_bytes);
        
        thread::sleep(time::Duration::from_millis(100));//at first was failing randomly, probably race condition with server thread
        send_data(num_bytes.clone(), addr);
    }

    #[test]
    fn remote_get_int() {
        let addr = get_server_addr("0065074.student.liacs.nl");
        let num:i32 = 1234567890;
        let num_serial = serde_cbor::to_vec(&num).expect("int is serializable");
        eprintln!("data: {:?}",num);
        eprintln!("serialized: {:?}",num_serial);

        let num_recv:i32 = receive_data(addr);
        assert_eq!(num,num_recv);
    }

    #[test]
    fn remote_send_request() {
        let addr = get_server_addr("0065074.student.liacs.nl");
        let data = RMIRequest::default();
        let data_serial = serde_cbor::to_vec(&data).expect("RMIRequest is serializable");
        
        thread::sleep(time::Duration::from_millis(10));
        send_data(data_serial, addr);
    }

    #[test]
    fn remote_get_request() {
        let addr = get_server_addr("0065074.student.liacs.nl");
        let req = RMIRequest::default();
        let req_recv: RMIRequest = receive_data(addr);
        assert_eq!(req,req_recv)
    }

}