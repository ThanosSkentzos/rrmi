use std::cell::RefCell;
use std::io::{Read, Write};
pub use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};

use crate::stub::{Deserialize, Serialize};

use crate::error::RMIError;
use crate::remote::RMIResult;
use crate::stub::{marshal, unmarshal};
use crate::transport::Transport;

pub fn send_data(data_serial: Vec<u8>, stream: &mut TcpStream) -> RMIResult<()> {
    let len = data_serial.len() as u32;
    let _ = stream.write_all(&len.to_be_bytes()).map_err(|e| {
        eprintln!("write len failed {e}");
        RMIError::TransportError(e.to_string())
    })?;
    let _ = stream.write_all(&data_serial).map_err(|e| {
        eprintln!("write data failed {e}");
        RMIError::TransportError(e.to_string())
    })?;
    let _ = stream.flush().map_err(|e| {
        eprintln!("flush failed {e}");
        RMIError::TransportError(e.to_string())
    })?;
    // eprintln!("tcp data sent");
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
#[allow(unused)]
pub struct TcpClient {
    server_addr: SocketAddr,
    stream: RefCell<TcpStream>,
    pub address: SocketAddr,
}

impl TcpClient {
    pub fn new(server_addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(server_addr).expect("Could not connect to server");
        stream.set_nodelay(true).unwrap();
        let address = stream
            .local_addr()
            .expect("Could not get stream address")
            .clone();
        let stream = RefCell::new(stream);
        Self {
            server_addr,
            stream,
            address,
        }
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
        let mut stream = self.stream.borrow_mut();
        send_data(request_serialized, &mut stream).map_err(|e| {
            eprintln!("send_data failed: {e:?}");
            e
        })?;
        // eprintln!("receive_data");
        let response_bytes = receive_data(&mut stream);
        // eprintln!("unmarshaling");
        let response: RES = unmarshal(&response_bytes)?;
        Ok(response)
    }
}