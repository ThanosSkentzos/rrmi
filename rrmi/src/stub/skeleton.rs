use std::sync::Arc;

use crate::remote::{RMIResult, RemoteObject};
use crate::stub::{marshal, unmarshal};
use crate::transport::utils::find_available_port_os;
use crate::transport::{RMIRequest, RMIResponse};
use crate::transport::{TcpStream, receive_data, send_data};

pub struct Skeleton {
    object: Arc<dyn RemoteObject>, // Arc because eventually we to listen from several ports
}

impl Skeleton {
    pub fn new(object: Arc<dyn RemoteObject>) -> Self {
        Skeleton { object }
    }
    pub fn listen(self: &Arc<Self>) -> RMIResult<u16> {
        let listener = find_available_port_os()?;
        let self_clone = Arc::clone(&self);
        let addr = listener.local_addr().expect("should have address");
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        if let Err(e) = self_clone.handle_connection(stream) {
                            eprintln!("Error: {e} when handling connection");
                        }
                    }
                    Err(e) => eprintln!("Transport error: {e}"),
                };
            }
        });
        Ok(addr.port())
    }

    fn handle_connection(&self, mut stream: TcpStream) -> RMIResult<()> {
        let request_bytes = receive_data(&mut stream);

        let request: RMIRequest = unmarshal(&request_bytes)?;
        let response = self.handle_request(request);
        let response_bytes = marshal(&response)?;

        send_data(response_bytes, &mut stream)
    }

    pub fn handle_request(&self, request: RMIRequest) -> RMIResponse {
        eprintln!("Skeleton got request {request:?}");
        match self
            .object
            .run(&request.method_name, request.serialized_args)
        {
            Ok(result) => RMIResponse::success(result),
            Err(e) => RMIResponse::error(format!("{e}")),
        }
    }
}

//#TODO tests
