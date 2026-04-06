use std::sync::Arc;

use crate::remote::{RMIResult, RemoteObject};
use crate::transport::utils::get_tcp_port;

pub struct Skeleton {
    object: Arc<dyn RemoteObject>, // Arc because eventually we to listen from several ports
}

impl Skeleton {
    pub fn new(object: Arc<dyn RemoteObject>) -> Self {
        Skeleton { object }
    }
    pub fn listen(self: &Arc<Self>) -> RMIResult<u16> {
        let listener = get_tcp_port()?;
        let self_clone = Arc::clone(&self);
        let addr = listener.local_addr().expect("Object should have address");
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        // eprintln!("Object received connection from {:?}", stream.peer_addr());
                        if let Err(e) = self_clone.object.run(&mut stream) {
                            eprintln!("Error: {e} when handling connection");
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
