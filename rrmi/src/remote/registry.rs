#[allow(non_camel_case_types)]
pub type RMI_ID = usize;
use super::{RemoteObject, RemoteRef};
use crate::error::RMIError;
use crate::stub::Skeleton;
use crate::transport::SocketAddr;
use crate::transport::utils::{get_addr, get_local_ips};

use rrmi_macros::remote_object;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
pub struct Registry {
    // a hashmap with all objects
    pub port: u16,
    objects: Arc<Mutex<HashMap<RMI_ID, Arc<Skeleton>>>>, // hashmap and objects should be thread safe
    names: Arc<Mutex<HashMap<String, RMI_ID>>>,
    next_id: Arc<AtomicUsize>,
}
#[remote_object]
impl Registry {
    fn new(port: u16) -> Self {
        Registry {
            port,
            objects: Arc::new(Mutex::new(HashMap::new())),
            names: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicUsize::new(1)), // keep 0 for itself
        }
    }

    #[allow(dead_code)]
    pub fn default() -> Registry {
        Registry::new(1099)
    }

    pub fn get_addr(&self, port: u16) -> SocketAddr {
        // this will be slower than just saving it
        let ips = get_local_ips().expect("Should be able to get local ip");
        let ip = ips[0]; //use 1st for now TODO handle eth or ib
        SocketAddr::new(ip, port)
    }
    pub fn remove(&self, name: &str) -> RMIResult<()> {
        let mut names = self.names.lock().unwrap();
        let id = names
            .get(name)
            .cloned()
            .ok_or(RMIError::NameNotFound(name.to_string()))?;
        names.remove(name);

        let mut objects = self.objects.lock().unwrap();
        let _sk = objects.remove(&id).ok_or(RMIError::ObjectNotFound(id))?;
        // todo!("make sure the object is also droped");
        // let left = objects.keys().count();
        // let strong = Arc::strong_count(&sk);
        // let weak = Arc::strong_count(&sk);
        // eprintln!("removed: now strong, weak = {strong},{weak} remaining: {left}");
        Ok(())
    }
    #[allow(dead_code)]
    pub fn remove_log(&self, name: &str) -> RMIResult<()> {
        eprintln!("removing {name}");
        self.remove(name)
    }

    pub fn get(&self, id: &RMI_ID) -> RMIResult<Arc<Skeleton>> {
        //! RMI_ID -> Skeleton | for server
        let objects = self.objects.lock().unwrap();
        objects
            .get(id)
            .cloned()
            .ok_or(RMIError::ObjectNotFound(*id))
    }
    #[remote]
    fn lookup(&self, name: &str) -> RMIResult<RemoteRef> {
        //! name -> remote ref | for client
        let names = self.names.lock().unwrap();
        let id = names
            .get(name)
            .ok_or(RMIError::NameNotFound(name.to_string()))?;
        let skeleton = self.get(id)?;
        let port = skeleton.listen()?;
        let addr = self.get_addr(port);
        Ok(RemoteRef { addr, id: *id })
    }

    #[allow(dead_code)]
    pub fn lookup_log(&self, name: &str) -> RMIResult<RemoteRef> {
        let res = self.lookup(name);
        match res.clone() {
            Ok(rref) => eprintln!(
                "Registry gives ref to skeleton listening at {:?}",
                rref.addr
            ),
            Err(_) => (),
        }
        res
    }

    #[remote]
    pub fn list(&self) -> RMIResult<Vec<String>> {
        let names: Vec<String> = self.names.lock().unwrap().keys().cloned().collect();
        match names.len() {
            0 => RMIResult::Err(RMIError::EmptyRegistry()),
            _ => RMIResult::Ok(names),
        }
    }

    pub fn bind(&self, name: &str, object: impl RemoteObject + 'static) -> RMI_ID {
        // bind a skelton to the registry
        //TODO: object is a skeleton
        // let skeleton = Arc::new(object);
        let skeleton = Arc::new(Skeleton::new(Arc::new(object)));
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.objects.lock().unwrap().insert(id, skeleton);
        self.names.lock().unwrap().insert(name.to_string(), id);
        eprintln!("Registered {id}: {name}");
        id
    }

    pub fn unbind(&self) {
        todo!()
        // switch variable to unbind
    }
}

pub fn create_registry(port: u16) -> Arc<Registry> {
    let reg = Arc::new(Registry::new(port));
    let port = reg.listen().expect("Registry should be able to listen");
    eprintln!("RMI Registry listening on {}", port);
    reg
}

pub fn get_registry(host: &str, port: u16) -> RegistryStub {
    let addr = get_addr(&host, port);
    let remote_ref_ref = RemoteRef::new(addr, 0);
    RegistryStub::new(remote_ref_ref)
    // todo!("to do this I need to ask the registry for its reference and treat it like a skeleton")
}

// Following code will be generated from the proc-macro
use ::rrmi::RMIResult;
use ::rrmi::stub::{Deserialize, Serialize, Stub};
use ::rrmi::transport::{TcpClient, TcpListener, TcpStream, Transport};
use rrmi::{marshal, receive_data, send_data, unmarshal};

#[derive(Serialize, Deserialize)]
pub enum RegistryRequest {
    Lookup { name: String },
    List,
}

#[derive(Serialize, Deserialize)]
pub enum RegistryResponse {
    Lookup(RMIResult<RemoteRef>),
    List(RMIResult<Vec<String>>),
}

impl Registry {
    pub fn listen(self: &Arc<Self>) -> RMIResult<u16> {
        // takes an arc reference to self Arc<Registry>
        // clone and move to a listening thread
        let listener = TcpListener::bind(("0.0.0.0", self.port))
            .map_err(|e| RMIError::TransportError(e.to_string()))?;
        let self_clone = Arc::clone(&self);
        let addr = listener.local_addr().expect("Registry should have address");
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        eprintln!("Registry received connection from {:?}", stream.peer_addr());
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
        let request: RegistryRequest = unmarshal(&request_bytes)?;

        let response: RegistryResponse = self.handle_request(request);

        let response_bytes = marshal(&response)?;
        send_data(response_bytes, &mut stream)
    }

    fn handle_request(&self, req: RegistryRequest) -> RegistryResponse {
        match req {
            RegistryRequest::Lookup { name } => RegistryResponse::Lookup(self.lookup(&name)),
            RegistryRequest::List => RegistryResponse::List(self.list()),
        }
    }
}

pub struct RegistryStub {
    remote: RemoteRef,
}
impl RegistryStub {
    pub fn new(remote: RemoteRef) -> Self {
        RegistryStub { remote }
    }

    pub fn lookup(&self, name: &str) -> RMIResult<Stub> {
        let transport = TcpClient::new(self.remote.addr);
        let req = RegistryRequest::Lookup {
            name: name.to_string(),
        };
        let resp: RegistryResponse = transport.send(req)?;
        match resp {
            RegistryResponse::Lookup(Ok(res)) => Ok(Stub::new(res)),
            _ => Err(RMIError::TransportError("Wrong response".to_string())),
        }
    }
    pub fn list(&self) -> RMIResult<Vec<String>> {
        let transport = TcpClient::new(self.remote.addr);
        let req = RegistryRequest::List {};
        let resp: RegistryResponse = transport.send(req)?;
        match resp {
            RegistryResponse::List(res) => res,
            _ => Err(RMIError::TransportError("Wrong response".to_string())),
        }
    }
}

impl RegistryStub {
    #[allow(dead_code)]
    fn lookup_log(&self, name: &str) -> RMIResult<Stub> {
        let res = self.lookup(name);
        match res.clone() {
            Ok(stub) => eprintln!(
                "RegistryStub returned stub for skeleton listening at {:?}",
                stub.get_ref()
            ),
            Err(_) => (),
        }
        res
    }
}
