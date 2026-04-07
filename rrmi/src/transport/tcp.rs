use std::io::{Read, Write};
pub use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};

use crate::stub::{Deserialize, Serialize};

use crate::error::RMIError;
use crate::remote::RMIResult;
use crate::stub::{marshal, unmarshal};
use crate::transport::Transport;

pub fn send_data(data_serial: Vec<u8>, stream: &mut TcpStream) -> RMIResult<()> {
    let len = data_serial.len() as u32;
    // eprintln!("tcp sending {len} bytes...");
    let _ = stream
        .write_all(&len.to_be_bytes())
        .map_err(|e| RMIError::TransportError(e.to_string()))?;
    let _ = stream
        .write_all(&data_serial)
        .map_err(|e| RMIError::TransportError(e.to_string()))?;
    let _ = stream
        .flush()
        .map_err(|e| RMIError::TransportError(e.to_string()))?;
    Ok(())
}

pub fn receive_data(stream: &mut TcpStream) -> Vec<u8> {
    let mut len_bytes = [0u8; 4];
    let _ = stream.read_exact(&mut len_bytes);
    let response_len = u32::from_be_bytes(len_bytes) as usize;

    // eprintln!("tcp reading response {response_len:?} bytes...");
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
        let request_serialized = marshal(&req)?;
        // eprintln!("send_data");
        let mut stream = TcpStream::connect(self.server_addr).unwrap();
        send_data(request_serialized, &mut stream)?; // return error to not block
        // eprintln!("receive_data");
        let response_bytes = receive_data(&mut stream);
        // eprintln!("unmarshaling");
        let response: RES = unmarshal(&response_bytes)?;
        Ok(response)
    }
}
#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    use crate::{transport::RMIRequest, utils::get_addr};
    static HOSTNAME_RECV: &str = "0065074.student.liacs.nl";
    static LOCAL_GET_SEND: u16 = 10999;
    static REMOTE_GET_SEND: u16 = 11000;

    #[test]
    #[ignore]
    fn liacs_ips() {
        let hostname = "0.0.0.0";
        get_addr(hostname, 1099);
        let hostname = "localhost";
        get_addr(hostname, 1099);
        let hostname = "0065074.student.liacs.nl";
        get_addr(hostname, 1099);
        let hostname = "0065073.student.liacs.nl";
        get_addr(hostname, 1099);
    }

    #[test]
    fn local_tcp_test() {
        let recv_handle = thread::spawn(|| {
            get_int_struct("localhost", LOCAL_GET_SEND);
        });
        thread::sleep(std::time::Duration::from_millis(100));
        send_int_struct("localhost", LOCAL_GET_SEND);
        recv_handle.join().expect("should be able to join");
    }

    #[test]
    #[ignore]
    fn remote_send() {
        send_int_struct(HOSTNAME_RECV, REMOTE_GET_SEND);
    }
    #[test]
    #[ignore]
    fn remote_recv() {
        get_int_struct(HOSTNAME_RECV, REMOTE_GET_SEND);
    }

    fn get_int_struct(hostname: &str, port: u16) {
        let num: i32 = 1234567890;
        eprintln!("data: {:?}", num);
        let addr = get_addr(hostname, port);
        // let mut stream = TcpStream::connect(addr).unwrap();
        let listener = TcpListener::bind(addr).expect("should be free");
        let (mut stream, _) = listener.accept().expect("should send");
        let bytes = receive_data(&mut stream);
        let num_recv: i32 = unmarshal(&bytes).expect("i32");
        assert_eq!(num_recv, num);

        let req = RMIRequest::default();
        let bytes = receive_data(&mut stream);
        let req_recv: RMIRequest = unmarshal(&bytes).expect("RMIRequest");
        assert_eq!(req_recv, req);
    }

    fn send_int_struct(hostname: &str, port: u16) {
        let addr = get_addr(hostname, port);
        let mut stream = TcpStream::connect(addr).unwrap();
        let int: i32 = 1234567890;
        let int_bytes = marshal(&int).expect("int is serializable");
        eprintln!("data: {:?}", int);
        eprintln!("serialized: {:?}", int_bytes);

        let _ = send_data(int_bytes.clone(), &mut stream);

        let request = RMIRequest::default();
        let request_bytes = marshal(&request).expect("RMIRequest is serializable");
        let _ = send_data(request_bytes, &mut stream);
    }
}
