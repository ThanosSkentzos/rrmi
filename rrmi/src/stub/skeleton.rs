use std::fmt::Debug;
use std::io::ErrorKind;
use std::sync::Arc;

#[cfg(feature = "tracing")]
use tracing::instrument;
#[cfg(feature = "tracing")]
use tracing::{Level, span};

use crate::remote::{RMIResult, RemoteObject};
use crate::transport::utils::get_tcp_socket_os;

pub struct Skeleton {
    object: Arc<dyn RemoteObject>, // Arc because eventually we to listen from several ports
}

impl Skeleton {
    pub fn new(object: Arc<dyn RemoteObject>) -> Self {
        Skeleton { object }
    }

    #[cfg_attr(feature = "tracing", instrument)]
    pub fn listen(&self) -> RMIResult<u16> {
        let listener = get_tcp_socket_os()?;
        let obj_clone = Arc::clone(&self.object);
        let object_name = obj_clone.name();
        let addr = listener
            .local_addr()
            .expect(&format!("{object_name}: does not have an address"));
        eprintln!("{object_name} uses address: {addr}");
        let port = addr.port();
        let name = format!("Skeleton{object_name}:{port}");
        let _handle_skeleton = std::thread::Builder::new().name(name).spawn(move || {
            // for stream in listener.incoming() {
            #[cfg(feature = "tracing")]
            let span = span!(Level::TRACE, "listen");
            #[cfg(feature = "tracing")]
            let _enter = span.enter();
            let stream = listener.accept();
            match stream {
                Ok((mut stream, _)) => {
                    eprintln!(
                        "{object_name} established connection with {:?}",
                        stream.peer_addr()
                    );
                    stream.set_nodelay(true).expect("Could not set NO_DELAY");
                    let mut buf = [0u8; 4];
                    loop {
                        #[cfg(feature = "tracing")]
                        let span = span!(Level::TRACE, "peek");
                        #[cfg(feature = "tracing")]
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
                        #[cfg(feature = "tracing")]
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
