#[allow(non_camel_case_types)]
pub type RMI_ID = usize;
use super::{RemoteObject, RemoteRef};
use crate::error::RMIError;
use crate::stub::Skeleton;
use crate::transport::SocketAddr;
use crate::transport::utils::{get_addr, get_local_ips};

// use rrmi_macros::remote_object;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(debug_assertions)]
use tracing::instrument;

#[derive(Debug)]
pub struct Registry {
    // a hashmap with all objects
    pub port: u16,
    objects: Arc<Mutex<HashMap<RMI_ID, Arc<Skeleton>>>>, // hashmap and objects should be thread safe
    names: Arc<Mutex<HashMap<String, RMI_ID>>>,
    next_id: Arc<AtomicUsize>,
}
// #[remote_object]
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

    #[cfg_attr(debug_assertions, instrument)]
    pub fn get_ip(&self) -> Result<IpAddr, ()> {
        let ips = get_local_ips().map_err(|e| eprintln!("Error getting local ip: {e:?}"));
        match ips {
            Ok(ips) => {
                let ip = ips[0];
                Ok(ip)
            }
            Err(_) => Err(()),
        }
    }

    #[cfg_attr(debug_assertions, instrument)]
    pub fn construct_addr(&self, port: u16) -> Result<SocketAddr, ()> {
        // this will be slower than just saving it
        let ips = get_local_ips().map_err(|e| eprintln!("Error getting local ip: {e:?}"));
        //TODO: handle multiple ips case
        match ips {
            Ok(ips) => {
                let ip = ips[0];
                Ok(SocketAddr::new(ip, port))
            }
            Err(_) => Err(()),
        }
    }

    #[cfg_attr(debug_assertions, instrument)]
    pub fn remove(&self, name: &str) -> RMIResult<()> {
        let mut names = self
            .names
            .lock()
            .expect("Registry: unable to get names lock");
        let id = names
            .get(name)
            .cloned()
            .ok_or(RMIError::NameNotFound(name.to_string()))?;
        names.remove(name);
        let mut objects = self
            .objects
            .lock()
            .expect("Registry: unable to get objects lock");
        let _sk = objects.remove(&id).ok_or(RMIError::ObjectNotFound(id))?;
        // todo!("make sure the object is also droped");
        // let left = objects.keys().count();
        // let strong = Arc::strong_count(&sk);
        // let weak = Arc::strong_count(&sk);
        // eprintln!("removed: now strong, weak = {strong},{weak} remaining: {left}");
        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument)]
    pub fn get(&self, id: &RMI_ID) -> RMIResult<Arc<Skeleton>> {
        //! RMI_ID -> Skeleton | for server
        let objects = self
            .objects
            .lock()
            .expect("Registry: unable to get objects lock");
        objects
            .get(id)
            .cloned()
            .ok_or(RMIError::ObjectNotFound(*id))
    }

    // #[remote]
    #[cfg_attr(debug_assertions, instrument)]
    pub fn lookup(&self, name: &str) -> RMIResult<RemoteRef> {
        //! name -> remote ref | for client
        let names = self
            .names
            .lock()
            .expect("Registry: unable to get names lock");
        let id = names
            .get(name)
            .ok_or(RMIError::NameNotFound(name.to_string()))?;
        let skeleton = self.get(id)?;
        let port = skeleton.listen()?;
        let addr = self
            .construct_addr(port)
            .map_err(|_| RMIError::TransportError("Cannot get local ip".to_string()))?;
        Ok(RemoteRef { addr, id: *id })
    }

    // #[remote]
    #[cfg_attr(debug_assertions, instrument)]
    pub fn list(&self) -> RMIResult<Vec<String>> {
        let names: Vec<String> = self
            .names
            .lock()
            .expect("Registry: unable to get names lock")
            .keys()
            .cloned()
            .collect();
        match names.len() {
            0 => RMIResult::Err(RMIError::EmptyRegistry()),
            _ => RMIResult::Ok(names),
        }
    }

    pub fn bind(&self, name: &str, object: impl RemoteObject + 'static) -> RMI_ID {
        // bind a skelton to the registry
        let skeleton = Arc::new(Skeleton::new(Arc::new(object)));
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.objects
            .lock()
            .expect("Registry: unable to get objects lock")
            .insert(id, skeleton);
        self.names
            .lock()
            .expect("Registry: unable to get names lock")
            .insert(name.to_string(), id);
        eprintln!("Registered {id}: {name}");
        id
    }

    pub fn unbind(&self) {
        todo!()
        // switch variable to unbind
    }
}
/// Creates and exports a Registry instance on the local host that accepts requests on the specified port.
///
/// Parameters:
/// port: `u16` the port on which the registry accepts requests
///
/// Returns:
/// the registry
///
/// ```
/// // create registry
/// use rrmi::{get_registry,create_registry};
/// let port: u16 = 1099;
/// let local = "localhost";
/// let reg = create_registry(port);
/// // check that is has the right port
/// assert_eq!(port,reg.port);
///
/// ```
#[cfg_attr(debug_assertions, instrument)]
pub fn create_registry(port: u16) -> Arc<Registry> {
    let reg = Arc::new(Registry::new(port));
    let port = reg.listen().expect("Registry: unable to start listening");
    eprintln!("RMI Registry listening on {}", port);
    reg
}

/// Returns a reference to the remote object Registry on the specified host and port
///
/// Parameters:
///
/// host: `&str` of host for the remote registry
///
/// port: `u16` port on which the registry accepts requests
///
/// Returns:
///
/// reference (a stub) to the remote object registry
/// ```
/// // create registry
/// use rrmi::{get_registry,create_registry};
/// let port: u16 = 1100;
/// let local = "localhost";
/// let reg = create_registry(port);
/// let ip = reg.get_ip().expect("should have address");
/// // access the registry
/// let reg_local = get_registry(local,1099);
/// let reg_remote = get_registry(&ip.to_string(),1099);
///
/// ```
//TODO: should i check connection and throw error?
#[cfg_attr(debug_assertions, instrument)]
pub fn get_registry(host: &str, port: u16) -> RegistryStub {
    let addr = get_addr(&host, port);
    let remote = RemoteRef::new(addr, 0);
    RegistryStub { remote }
}

use ::rrmi::RMIResult;
use ::rrmi::transport::TcpListener;

impl Registry {
    #[cfg_attr(debug_assertions, instrument)]
    pub fn listen(self: &Arc<Self>) -> RMIResult<u16> {
        // takes an arc reference to self Arc<Registry>
        // clone and move to a listening thread
        let listener = TcpListener::bind(("0.0.0.0", self.port)).map_err(|e| {
            eprintln!("Registry Error: cannot bind port {e}");
            RMIError::TransportError(e.to_string())
        })?;
        let self_clone = Arc::clone(&self);
        let addr = listener
            .local_addr()
            .expect("Registry: does not have an address");
        let _handle_registry = std::thread::Builder::new()
            .name("Registry".to_string())
            .spawn(move || {
                for stream in listener.incoming() {
                    match stream {
                        Ok(mut stream) => {
                            eprintln!("Registry received connection from {:?}", stream.peer_addr());
                            if let Err(e) = self_clone.run(&mut stream) {
                                eprintln!("Error: {e} when handling connection");
                            }
                        }
                        Err(e) => eprintln!("Transport error: {e}"),
                    };
                }
            })
            .expect("Registry thread failed");
        Ok(addr.port())
    }
}

// Remote object code
// this could also be generated from the macro
use ::rrmi::stub::{Deserialize, Serialize, Stub};
use ::rrmi::transport::{TcpClient, TcpStream, Transport};
use rrmi::{marshal, receive_data, send_data, unmarshal};

#[derive(Serialize, Deserialize, Debug)]
pub enum RegistryRequest {
    Lookup { name: String },
    List,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RegistryResponse {
    Lookup(RMIResult<RemoteRef>),
    List(RMIResult<Vec<String>>),
}

impl RemoteObject for Registry {
    fn run(&self, stream: &mut TcpStream) -> RMIResult<()> {
        self.handle_connection(stream)
    }
    fn name(&self) -> &'static str {
        "Registry"
    }
}
#[allow(dead_code)]
impl Registry {
    #[cfg_attr(debug_assertions, instrument)]
    fn handle_connection(&self, stream: &mut TcpStream) -> RMIResult<()> {
        stream.set_nodelay(true).expect("Could not set NO_DELAY");
        let request_bytes = receive_data(stream);
        let request: RegistryRequest = unmarshal(&request_bytes)?;
        let response: RegistryResponse = self.handle_request(request);
        let response_bytes = marshal(&response)?;
        send_data(response_bytes, stream)
    }
    #[cfg_attr(debug_assertions, instrument)]
    fn handle_request(&self, req: RegistryRequest) -> RegistryResponse {
        match req {
            RegistryRequest::Lookup { name } => RegistryResponse::Lookup(self.lookup(&name)),
            RegistryRequest::List => RegistryResponse::List(self.list()),
        }
    }
}

#[derive(Debug)]
pub struct RegistryStub {
    remote: RemoteRef,
}
impl RegistryStub {
    pub fn new(remote: RemoteRef) -> Self {
        RegistryStub { remote }
    }

    #[cfg_attr(debug_assertions, instrument)]
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
    #[cfg_attr(debug_assertions, instrument)]
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
