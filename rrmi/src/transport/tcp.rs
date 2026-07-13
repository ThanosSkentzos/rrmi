use std::cell::RefCell;
use std::fmt::Debug;
use std::io::{ErrorKind, IoSlice, Read, Write};
pub use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;

use crate::TransportServer;
use crate::stub::{Deserialize, Serialize};

use crate::error::RMIError;
use crate::remote::{RMIResult, RemoteObject};
use crate::stub::{marshal, unmarshal};
use crate::transport::TransportClient;
use crate::utils::{get_my_hostname, get_tcp_socket_os};

#[cfg(feature = "tracing")]
use tracing::instrument;
#[cfg(feature = "tracing")]
use tracing::{Level, span};

#[cfg_attr(feature = "tracing", instrument)]
pub fn send_bytes(data_serial: Vec<u8>, stream: &mut TcpStream) -> RMIResult<()> {
    _send_data_ioslice(data_serial, stream)
}

pub fn _send_data_separate(data_serial: Vec<u8>, stream: &mut TcpStream) -> RMIResult<()> {
    let len: [u8;8] = data_serial.len().to_be_bytes();
    let _ = stream.write_all(&len).map_err(|e| {
        eprintln!("write len failed {e}");
        RMIError::TransportError(e.to_string())
    })?;
    let _ = stream.write_all(&data_serial).map_err(|e| {
        eprintln!("write data failed {e}");
        RMIError::TransportError(e.to_string())
    })?;
    Ok(())
}

pub fn _send_data_separate_flush(data_serial: Vec<u8>, stream: &mut TcpStream) -> RMIResult<()> {
    let len: [u8; 8] = data_serial.len().to_be_bytes();
    let _ = stream.write_all(&len).map_err(|e| {
        eprintln!("write len failed {e}");
        RMIError::TransportError(e.to_string())
    })?;
    let _ = stream.write_all(&data_serial).map_err(|e| {
        eprintln!("write data failed {e}");
        RMIError::TransportError(e.to_string())
    })?;
    // flush should do nothing for TCP cause its not buffered
    let _ = stream.flush().map_err(|e| {
        eprintln!("flush failed {e}");
        RMIError::TransportError(e.to_string())
    })?;
    Ok(())
}
pub fn _send_data_combined(data_serial: Vec<u8>, stream: &mut TcpStream) -> RMIResult<()> {
    let len: [u8; 8] = (data_serial.len() as u64).to_be_bytes();
    let mut buf = Vec::with_capacity(4 + data_serial.len());
    buf.extend_from_slice(&len);
    buf.extend_from_slice(&data_serial);
    let _ = stream
        .write_all(&buf)
        .map_err(|e| RMIError::TransportError(e.to_string()))?;
    Ok(())
}
pub fn _send_data_ioslice(data_serial: Vec<u8>, stream: &mut TcpStream) -> RMIResult<()> {
    //TODO: find a way of sending just the data, not the size first
    let len: [u8; 8] = (data_serial.len() as u64).to_be_bytes();

    let bufs = &[IoSlice::new(&len), IoSlice::new(&data_serial)];
    let _ = stream
        .write_vectored(bufs)
        .map_err(|e| RMIError::TransportError(e.to_string()))?;
    Ok(())
}

#[cfg_attr(feature = "tracing", instrument)]
pub fn receive_bytes(stream: &mut TcpStream) -> RMIResult<Vec<u8>> {
    let mut len: [u8; 8] = [0u8; 8];
    stream.read_exact(&mut len).map_err(|e| match e.kind() {
        ErrorKind::UnexpectedEof => RMIError::TransportError("connection closed".into()),
        _ => RMIError::TransportError(e.to_string()),
    })?;
    let response_len = u64::from_be_bytes(len);

    const MAX_SIZE: u64 = 2_u64.pow(36);
    if response_len > MAX_SIZE {
        return Err(RMIError::TransportError(format!(
            "message with len {} exceeded maximum size of {}",
            response_len, MAX_SIZE
        )));
    }

    // eprintln!("tcp reading response {response_len:?} bytes...");
    //TODO how can I avoid reallocation here
    let mut bytes = vec![0u8; response_len as usize];
    stream.read_exact(&mut bytes).map_err(|e| match e.kind() {
        ErrorKind::UnexpectedEof => RMIError::TransportError("connection closed".into()),
        _ => RMIError::TransportError(e.to_string()),
    })?;
    Ok(bytes)
}

#[allow(unused)]
#[derive(Debug)]
pub struct TcpClient {
    server_addr: SocketAddr,
    stream: RefCell<TcpStream>,
    pub address: SocketAddr,
}

impl TcpClient {
    pub fn new(server_addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(server_addr).expect("Could not connect to server");
        let hostname = get_my_hostname();
        eprintln!(
            "{hostname}: TCP client connected to remote {} using {}",
            stream.peer_addr().unwrap(),
            stream.local_addr().unwrap()
        );
        stream.set_nodelay(true).expect("Could not set NO_DELAY");
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
#[cfg(feature = "tracing")]
impl TransportClient for TcpClient {
    fn send_req<
        REQ: Serialize + for<'de> Deserialize<'de> + Debug,
        RES: Serialize + for<'de> Deserialize<'de> + Debug,
    >(
        &self,
        req: REQ,
    ) -> RMIResult<RES> {
        // eprintln!("marshaling");
        let request_serialized = marshal(&req)?;
        // eprintln!("send_data");
        let mut stream = self.stream.borrow_mut();
        send_bytes(request_serialized, &mut stream).map_err(|e| {
            eprintln!("send_data failed: {e:?}");
            e
        })?;
        // eprintln!("receive_data");
        let response_bytes = receive_bytes(&mut stream).expect("Message exceeded maximum size");
        // eprintln!("unmarshaling");
        let response: RES = unmarshal(&response_bytes)?;
        Ok(response)
    }
}

#[cfg(not(feature = "tracing"))]
impl TransportClient for TcpClient {
    fn send_req<
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
        send_bytes(request_serialized, &mut stream).map_err(|e| {
            eprintln!("send_data failed: {e:?}");
            e
        })?;
        // eprintln!("receive_data");
        let response_bytes = receive_bytes(&mut stream).expect("Message exceeded maximum size");
        // eprintln!("unmarshaling");
        let response: RES = unmarshal(&response_bytes)?;
        Ok(response)
    }
}

pub struct TcpServer {
    listener: TcpListener,
    obj: Arc<dyn RemoteObject>,
}

impl TransportServer for TcpServer {
    fn new(obj: Arc<dyn RemoteObject>) -> Self {
        let listener = get_tcp_socket_os().expect("Was unable to get address");
        Self { listener, obj }
    }

    fn get_address(&self) -> SocketAddr {
        self.listener
            .local_addr()
            .expect(&format!("{}: does not have an address", self.obj.name()))
    }

    fn listen(&self) {
        let stream = self.listener.accept();

        match stream {
            Ok((stream, peer)) => {
                eprintln!(
                    "{} established connection with {:?}",
                    self.obj.name(),
                    stream.peer_addr()
                );
                stream.set_nodelay(true).expect("Could not set NO_DELAY");
                self.receive_loop(stream, peer);
            }
            Err(e) => eprintln!("Transport error:{e}"),
        };
    }
}
impl TcpServer {
    fn receive_loop(&self, mut stream: TcpStream, peer: SocketAddr) {
        let mut buf = [0u8; 4];
        loop {
            #[cfg(feature = "tracing")]
            let span = span!(Level::TRACE, "peek");
            #[cfg(feature = "tracing")]
            let _enter = span.enter();

            let num_bytes = stream.peek(&mut buf);
            match num_bytes {
                Ok(0) => {
                    eprintln!("{:?}: Connection to {peer} closed.", self.obj.name());
                    break;
                }
                // Ok(1) => {//handle reference moving to new peer
                //     let new_peer;
                //     (stream, new_peer) = self.listener.accept().expect("todo handle error");
                //     if new_peer == peer {// just reconnect
                //     epritnln!("Reconnected to {peer}");
                //     }
                //     let mut code = [0u8; 1];
                //     stream
                //         .read_exact(&mut code)
                //         .expect("Could not read tcp buffer");
                //     match code[0] {
                //         42 => continue,
                //         _ => {
                //             eprintln!(
                //                 "Invalid code on connection stealing.\nClosing connection..."
                //             );
                //             break;
                //         }
                //     }
                // }
                Ok(_) => (), //valid connection
                Err(e) => match e.kind() {
                    ErrorKind::ConnectionReset | ErrorKind::BrokenPipe => {
                        eprintln!("Connection closed due to error: {e}")
                    }
                    _k => eprintln!("Connection error {e:?}"),
                },
            }

            #[cfg(feature = "tracing")]
            drop(_enter);

            match self.obj.handle_connection(&mut stream) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "{:?} Connection closed when running: {e}",
                        stream.peer_addr()
                    );
                    break;
                }
            }
        }
    }
}
