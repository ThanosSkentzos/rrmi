use std::fmt::Debug;
use std::io::ErrorKind;
use std::sync::Arc;

#[cfg(debug_assertions)]
use tracing::{Level, span};

use crate::RMIError;
use crate::remote::{RMIResult, RemoteObject};
use crate::transport::utils::get_tcp_socket_os;

pub struct Skeleton {
    object: Arc<dyn RemoteObject>, // Arc because eventually we to listen from several ports
}

impl Skeleton {
    pub fn new(object: Arc<dyn RemoteObject>) -> Self {
        Skeleton { object }
    }

    pub fn listen(&self) -> RMIResult<u16> {
        let listener = get_tcp_socket_os()?;
        let obj_clone = Arc::clone(&self.object);
        let object_name = obj_clone.name();
        let addr = listener
            .local_addr()
            .expect(&format!("{object_name}: does not have an address"));
        eprintln!("{object_name} uses address: {addr}");
        let name = format!("Skeleton{object_name}");
        let _handle_skeleton = std::thread::Builder::new().name(name).spawn(move || {
            // for stream in listener.incoming() {
            #[cfg(debug_assertions)]
            let span = span!(Level::TRACE, "listen");
            #[cfg(debug_assertions)]
            let _enter = span.enter();
            let stream = listener.accept();
            match stream {
                Ok((mut stream, _)) => {
                    eprintln!(
                        "{object_name} established connection with {:?}",
                        stream.peer_addr()
                    );
                    stream
                        .set_nodelay(true)
                        .map_err(|e| RMIError::TransportError(e.to_string()))
                        .expect("Could not set NO_DELAY");
                    let mut buf = [0u8; 4];
                    loop {
                        #[cfg(debug_assertions)]
                        let span = span!(Level::TRACE, "peek");
                        #[cfg(debug_assertions)]
                        let _enter = span.enter();
                        match stream.peek(&mut buf) {
                            Ok(0) => {
                                eprintln!("{:?}: Connection closed.", obj_clone.name());
                                break;
                            }
                            Ok(_) => (),
                            Err(e) => match e.kind() {
                                ErrorKind::ConnectionReset | ErrorKind::BrokenPipe => {
                                    eprintln!("Connection closed due to error: {e}")
                                }
                                _k => eprintln!("Connection error {e:?}"),
                            },
                        };
                        #[cfg(debug_assertions)]
                        drop(_enter);
                        match obj_clone.run(&mut stream) {
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
                Err(e) => eprintln!("Transport error: {e}"),
            };
            // }
        });
        Ok(addr.port())
    }
}

impl Debug for Skeleton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Skeleton[{:?}]", self.object.name())
    }
}
//#TODO tests
