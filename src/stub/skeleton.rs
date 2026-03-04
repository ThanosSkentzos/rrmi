use std::net::{TcpStream};
use std::sync::Arc;

use crate::error::RMIError;
use crate::transport::utils::find_available_port;
use std::io::{Read, Write};
use crate::remote::{RMIResult, RemoteObject};
use crate::transport::{RMIRequest, RMIResponse};

pub struct Skeleton{
    object: Arc<dyn RemoteObject>,// Arc because eventually we to listen from several ports
}

impl Skeleton{
    pub fn new(object:Arc<dyn RemoteObject>) -> Self{
        Skeleton {object}
    }

    pub fn handle_request(
        &self,
        request: RMIRequest,
    ) -> RMIResponse{
        match self.object.run(&request.method_name, request.serialized_args){
            Ok(result) => RMIResponse::success(result),
            Err(e) => RMIResponse::error(format!("{e}")),
        }
    }
 
    pub fn listen(self: &Arc<Self>) -> RMIResult<u16>{
        let taken: Vec<u16> = vec![1099];
        let (listener,port) = find_available_port(&taken)?;
        let self_clone = Arc::clone(&self);
        std::thread::spawn(move ||{
            eprintln!("Skeleton listening at {port}");
            for stream in listener.incoming(){
                match stream{
                    Ok(stream) => {
                        if let Err(e) = self_clone.handle_connection(stream){
                            eprintln!("Error: {e} when handling connection");
                        }
                    }
                    Err(e) => eprintln!("Transport error: {e}"),
                };
            }
        });
        Ok(port)
    }

    fn handle_connection(&self, mut stream:TcpStream)-> RMIResult<()>{
        let mut len_bytes = [0u8; 4];
        let _ = stream.read_exact(&mut len_bytes)
            .map_err(|e| RMIError::TransportError(e.to_string()))?;
        
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut request_bytes = vec![0u8;len];
        stream.read_exact(&mut request_bytes)
            .map_err(|e| RMIError::TransportError(e.to_string()))?;

        let request: RMIRequest = serde_cbor::from_slice(&request_bytes)
            .map_err(|e| RMIError::DeserializationError(e.to_string()))?;
        eprintln!("Skeleton got request {request:?}");
        let response = self.handle_request(request);
        eprintln!("Skeleton response {response:?}");
        let response_bytes = serde_cbor::to_vec(&response)
            .map_err(|e| RMIError::SerializationError(e.to_string()))?;
        let len = response_bytes.len() as u32;

        stream.write_all(&len.to_be_bytes()).map_err(|e| RMIError::TransportError(e.to_string()))?;
        stream.write_all(&response_bytes).map_err(|e| RMIError::TransportError(e.to_string()))?;
        stream.flush().map_err(|e| RMIError::TransportError(e.to_string()))?;

        eprintln!("Response sent.");
        Ok(())
    }
}

//#TODO tests