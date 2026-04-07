use std::sync::Arc;

use crate::remote::{RMIResult, RemoteObject};
use crate::transport::utils::get_tcp_socket;

pub struct Skeleton {
    object: Arc<dyn RemoteObject>, // Arc because eventually we to listen from several ports
}

impl Skeleton {
    pub fn new(object: Arc<dyn RemoteObject>) -> Self {
        Skeleton { object }
    }
    pub fn listen(self: &Arc<Self>) -> RMIResult<u16> {
        let listener = get_tcp_socket()?;
        let self_clone = Arc::clone(&self);
        let object_name = self.object.name();
        let addr = listener
            .local_addr()
            .expect(&format!("{object_name}: does not have an address"));
        eprintln!("{object_name} uses address: {addr}");
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        eprintln!(
                            "{object_name} received connection from {:?}",
                            stream.peer_addr()
                        );
                        if let Err(e) = self_clone.object.run(&mut stream) {
                            eprintln!("{object_name} Error: {e} when handling connection");
                        }
                    }
                    Err(e) => eprintln!("Transport error: {e}"),
                };
            }
        });
        Ok(addr.port())
    }
}

//#TODO tests
