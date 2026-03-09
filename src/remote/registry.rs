pub type RMI_ID = usize;
use super::{RMIResult, RemoteObject, RemoteRef};
use crate::error::RMIError;
use crate::stub::{Skeleton, Stub};
use crate::transport::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, receive_data, send_data};
use crate::transport::{TcpClient, Transport, utils};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct Registry {
    // a hashmap with all objects
    port: u16,
    objects: Arc<Mutex<HashMap<RMI_ID, Arc<Skeleton>>>>, // hashmap and objects should be thread safe
    names: Arc<Mutex<HashMap<String, RMI_ID>>>,
    next_id: Arc<AtomicUsize>,
}

impl Registry {
    fn new(port: u16) -> Self {
        Registry {
            port,
            objects: Arc::new(Mutex::new(HashMap::new())),
            names: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicUsize::new(1)), // keep 0 for itself
        }
    }

    fn default() -> Registry {
        Registry::new(1099)
    }

    fn get_addr(&self, port: u16) -> SocketAddr {
        // this will be slower than just saving it
        let ip = local_ip_address::local_ip().expect("Should be able to get local ip");
        SocketAddr::new(ip, port)
    }

    fn remove(&self, name: &str) -> RMIResult<()> {
        let mut names = self.names.lock().unwrap();
        let id = names
            .get(name)
            .cloned()
            .ok_or(RMIError::NameNotFound(name.to_string()))?;
        names.remove(name);

        let mut objects = self.objects.lock().unwrap();
        let sk = objects.remove(&id).ok_or(RMIError::ObjectNotFound(id))?;
        // todo!("make sure the object is also droped");
        // let left = objects.keys().count();
        // let strong = Arc::strong_count(&sk);
        // let weak = Arc::strong_count(&sk);
        // eprintln!("removed: now strong, weak = {strong},{weak} remaining: {left}");
        Ok(())
    }

    fn remove_log(&self, name: &str) -> RMIResult<()> {
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

    fn lookup_log(&self, name: &str) -> RMIResult<RemoteRef> {
        let res = self.lookup(name);
        match res.clone() {
            Ok(rref) => eprintln!("Registry gives ref to skeleton listening at {:?}", rref.addr),
            Err(_) => (),
        }
        res
    }

    pub fn list(&self) -> RMIResult<Vec<String>> {
        let names: Vec<String> = self.names.lock().unwrap().keys().cloned().collect();
        match names.len() {
            0 => RMIResult::Err(RMIError::EmptyRegistry()),
            _ => RMIResult::Ok(names),
        }
    }

    pub fn bind(&self, name: &str, object: impl RemoteObject + 'static) -> RMI_ID {
        // bind a skelton to the registry
        let skeleton = Arc::new(Skeleton::new(Arc::new(object)));
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.objects.lock().unwrap().insert(id, skeleton);
        self.names.lock().unwrap().insert(name.to_string(), id);
        eprintln!("Registered {id}: {name}");
        id
    }

    pub fn listen(self) -> RMIResult<Arc<Registry>> {
        let socket = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str("0.0.0.0").expect("0.0.0.0 should pass")),
            self.port,
        );
        let listener =
            TcpListener::bind(socket).map_err(|e| RMIError::TransportError(e.to_string()))?;
        eprintln!("RMI Registry listening on {}", socket);
        let arc_reg = Arc::new(self);
        let arc_reg_clone = Arc::clone(&arc_reg);
        std::thread::spawn(move || {
            // bind separate port for each remote object + client (1 per client)
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        if let Err(e) = arc_reg_clone.handle_connection(stream) {
                            eprintln!("Error: {e} when handling connection");
                        }
                    }
                    Err(e) => eprintln!("Transport error: {e}"),
                };
            }
        });
        Ok(arc_reg)

        // check variable to unbind
        // gracefull shutdown or kill?
    }

    pub fn unbind(&self) {
        todo!()
        // switch variable to unbind
    }

    fn handle_connection(&self, mut stream: TcpStream) -> RMIResult<()> {
        let request_bytes = receive_data(&mut stream);

        let request: RegistryRequest = serde_cbor::from_slice(&request_bytes)
            .map_err(|e| RMIError::DeserializationError(e.to_string()))?;
        let response: RegistryResponse = self.handle_request(request);

        let response_bytes = serde_cbor::to_vec(&response)
            .map_err(|e| RMIError::SerializationError(e.to_string()))?;

        send_data(response_bytes,&mut stream)
    }

    fn handle_request(&self, req: RegistryRequest) -> RegistryResponse {
        match req {
            RegistryRequest::Lookup { name } => RegistryResponse::Lookup(self.lookup(&name)),
            RegistryRequest::List => RegistryResponse::List(self.list()),
        }
    }
}

pub fn create_registry(port: u16) -> Arc<Registry> {
    let reg = Registry::new(port);
    let reg = reg.listen().expect("Registry should be able to listen");
    reg
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
        match resp{
            RegistryResponse::Lookup(Ok(res))=> Ok(Stub::new(res)),
            _ => Err(RMIError::TransportError("Wrong response".to_string())),
        }
    }

    fn lookup_log(&self, name: &str) -> RMIResult<Stub> {
        let res = self.lookup(name);
        match res.clone() {
            Ok(stub) => eprintln!("RegistryStub returned stub for skeleton listening at {:?}", stub.get_ref()),
            Err(_) => (),
        }
        res
    }

    pub fn list(&self) -> RMIResult<Vec<String>> {
        let transport = TcpClient::new(self.remote.addr);
        let req = RegistryRequest::List {};
        let resp: RegistryResponse = transport.send(req)?;
        match resp{
            RegistryResponse::List(res)=> res,
            _ => Err(RMIError::TransportError("Wrong response".to_string())),
        }
    }
}

pub fn get_registry(host: &str, port: u16) -> RegistryStub {
    let addr = utils::get_server_addr(&host, port);
    let remote_ref_ref = RemoteRef::new(addr, 0);
    RegistryStub::new(remote_ref_ref)
    // todo!("to do this I need to ask the registry for its reference and treat it like a skeleton")
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        remote::MockRemoteObject,
        stub::{RemoteTrait, Stub},
    };
    use core::{panic, time};
    use std::{thread, time::Duration};
    use local_ip_address::local_ip;
    use threadpool::ThreadPool;

    static POPUL_PORT:u16 = 10996;
    static BIND_PORT:u16 = 10997;
    static LOCAL_PORT:u16 = 10998;
    static REMOTE_TEST_PORT:u16 = 12345;
    static REMOTE_HOST: &str = "0065074.student.liacs.nl";

    #[test]
    fn addr() {
        let reg = Registry::default();
        let port = reg.port;
        let addr = reg.get_addr(port);
        assert_eq!(
            addr,
            SocketAddr::new(local_ip().expect("Should be able to get ip"), port)
        )
    }

    #[test]
    fn populate_clear() {
        let reg = create_registry(POPUL_PORT);
        let reg = Arc::new(Mutex::new(reg));
        let pool = ThreadPool::new(2);
        let jobs = 10;
        let per_thread = 42;

        //REGISTER PHASE
        for thread in 0..jobs {
            let r = Arc::clone(&reg);
            pool.execute(move || {
                for n in 0..per_thread {
                    let name = format!("{thread}-{n}");
                    let guard = r.lock().unwrap();
                    guard.bind(&name, MockRemoteObject::silent());
                    drop(guard);
                }
            });
        }

        std::thread::sleep(time::Duration::from_millis(100));
        let num_objects = reg.lock().unwrap().list().unwrap().len();
        eprintln!("Num objects after populating {}", num_objects);
        assert_eq!(num_objects, jobs * per_thread);

        // DEREGISTER PHASE
        for thread in 0..jobs {
            let r = Arc::clone(&reg);
            pool.execute(move || {
                for n in 0..per_thread {
                    let guard = r.lock().unwrap();
                    let name = format!("{thread}-{n}");
                    guard.remove(&name).expect("should still have this process");
                    drop(guard);
                }
            });
        }

        std::thread::sleep(time::Duration::from_millis(100));
        let names = reg.lock().unwrap().list();

        match names {
            Result::Err(RMIError::EmptyRegistry()) => (),
            _ => panic!(),
        }
        // assert_eq!(names.err(), Option::Some(RMIError::EmptyRegistry()));
    }

    #[test]
    fn bind_lookup_list_remove() {
        let reg = create_registry(BIND_PORT);
        let rmt_reg = get_registry("localhost", BIND_PORT);

        let verbose = MockRemoteObject::verbose();
        let silent = MockRemoteObject::silent();
        reg.bind("verbose", verbose);
        reg.bind("silent", silent);

        let remote = reg.lookup_log("silent").expect("silent should be in");
        let remote = reg.lookup_log("verbose").expect("verbose should be in");

        let l = reg.list().expect("two already in");
        let l_rmt = rmt_reg.list().expect("same");
        eprintln!("local: {:?} vs remote: {:?}", l,l_rmt);
        reg.remove_log("verbose").expect("still in");

        let l = reg.list().expect("one still in");
        let l_rmt = rmt_reg.list().expect("same");
        eprintln!("local: {:?} vs remote: {:?}", l, l_rmt);
        reg.remove_log("silent").expect("still in");

        match reg.list() {
            Ok(_) => panic!("should not have any other objects"),
            Err(RMIError::EmptyRegistry()) => (),
            Err(_) => panic!("should return EmptyRegistry error"),
        };
    }

    #[test]
    fn local_listen() {
        let obj_verbose = MockRemoteObject::verbose();
        let args = vec![42; 2];
        let res_expected = args.clone();
        eprintln!("args: {args:?}");

        eprintln!("reg preparation");
        let reg = create_registry(LOCAL_PORT);
        reg.bind("verbose", obj_verbose);
        let rmt_reg = get_registry("localhost", LOCAL_PORT);
        let stb = rmt_reg.lookup_log("verbose").expect("verbose should be in");

        eprintln!("Stub: {stb:?}");
        //NEED TO KNOW THE RETURN TYPE
        let res: RMIResult<Vec<u8>> = stb.run_stub(args.clone());
        assert_eq!(res_expected, res.clone().unwrap());
        eprintln!("result: {res:?} matched expected\n\n");

        let obj2 = MockRemoteObject::verbose();
        let args2 = "I'm here too!";
        let sargs2 =
            serde_cbor::to_vec(&args2)
            .map_err(|e| RMIError::SerializationError(e.to_string()))
            .expect("should be able to serialize");
        let resp2 = obj2
            .run("locally method_name", sargs2)
            .expect("Mock object returns the args");
        let res2_expected: String = serde_cbor::from_slice(&resp2).expect("should be able to deserialize");
        reg.bind("second", obj2);
        let rmt2 = reg.lookup_log("second").expect("second should be in");
        let stb2 = Stub::new(rmt2);

        let res2: RMIResult<String> = stb2.run_stub(args2.clone());
        eprintln!("result: {res2:?} matched expected\n\n");
        assert_eq!(res2.unwrap(), res2_expected);
    }

    #[test]
    fn remote_skel(){
        // assume it runs on 0065074.student.liacs.nl
        let reg = create_registry(REMOTE_TEST_PORT);
        let obj_verbose = MockRemoteObject::verbose();
        reg.bind("verbose", obj_verbose);
        // how to block
        thread::sleep(Duration::from_secs(5));
    }

    #[test]
    fn remote_stub(){
        // runs after remote_listen on 00650??.student.liacs.nl
        let reg = get_registry(REMOTE_HOST, REMOTE_TEST_PORT);
        let stub = reg.lookup("verbose").expect("should work");
        let resp:RMIResult<Vec<u8>> = stub.run_stub(vec![42;2]);
        println!("{resp:?}")
    }
}
