pub type RMI_ID = usize;
use crate::error::RMIError;
use crate::stub::Skeleton;
use super::{RMIResult, RemoteObject, RemoteRef};

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize,Ordering};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

pub struct Registry{// a hashmap with all objects
    port: u16,
    objects: Arc<Mutex<HashMap<RMI_ID,Arc<Skeleton>>>>,// hashmap and objects should be thread safe
    names: Arc<Mutex<HashMap<String,RMI_ID>>>,
    next_id: Arc<AtomicUsize>,
}

impl Registry{
    pub fn new(port: u16) -> Self{
        Registry {
            port,
            objects: Arc::new(Mutex::new(HashMap::new())),
            names: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicUsize::new(1)),// keep 0 for itself
        }
    }

    pub fn default()-> Registry{
        Registry::new(1099)
    }

    fn get_addr(&self,port:u16)-> SocketAddr{
        // this will be slower than just saving it
        let ip = local_ip_address::local_ip().expect("Should be able to get local ip");
        SocketAddr::new(ip, port)
    }

    pub fn remove(&self, name:&str) -> RMIResult<()>{
        let mut names = self.names.lock().unwrap();
        let id = names.get(name).cloned().ok_or(RMIError::NameNotFound(name.to_string()))?;
        names.remove(name);

        let mut objects = self.objects.lock().unwrap();
        let sk = objects.remove(&id)
            .ok_or(RMIError::ObjectNotFound(id))?;
        // todo!("make sure the object is also droped");
        // let left = objects.keys().count();
        // let strong = Arc::strong_count(&sk);
        // let weak = Arc::strong_count(&sk);
        // eprintln!("removed: now strong, weak = {strong},{weak} remaining: {left}");
        Ok(())
    }

    pub fn get(&self, id: &RMI_ID) -> RMIResult<Arc<Skeleton>>{
        //! RMI_ID -> Skeleton | for server
        let objects = self.objects.lock().unwrap();
        objects.get(id)
            .cloned()
            .ok_or(RMIError::ObjectNotFound(*id))
    }

    pub fn lookup(&self,name:&str) -> RMIResult<RemoteRef>{
        //! name -> remote ref | for client
        let names = self.names.lock().unwrap();
        let id = names.get(name)
            .ok_or(RMIError::NameNotFound(name.to_string()))?;
        eprintln!("Registry spawning new thread for {name} | id: {id}");
        let skeleton = self.get(id)?;
        let port = skeleton.listen()?;
        let addr = self.get_addr(port);

        Ok(RemoteRef { addr, id: *id })
    }

    pub fn list(&self) -> RMIResult<Vec<String>>{
        let names: Vec<String> = self.names.lock().unwrap().keys().cloned().collect();
        match names.len(){
            0 => RMIResult::Err(RMIError::EmptyRegistry()),
            _ => RMIResult::Ok(names)
        }
    }

    pub fn bind(&self,name:&str,object: impl RemoteObject + 'static) -> RMI_ID{
        // bind a skelton to the registry
        let skeleton = Arc::new(Skeleton::new(Arc::new(object)));
        let id= self.next_id.fetch_add(1,Ordering::Relaxed);
        self.objects.lock().unwrap().insert(id, skeleton);
        self.names.lock().unwrap().insert(name.to_string(), id);
        eprintln!("Registered {id}: {name}");
        id
    }

    pub fn listen(self: &Arc<Self>) -> RMIResult<()>{
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str("0.0.0.0").expect("0.0.0.0 should pass")),self.port);
        let listener = TcpListener::bind(socket)
            .map_err(|e| RMIError::TransportError(e.to_string()))?;
        eprintln!("RMI Registry listening on {}", socket);

        let self_clone = Arc::clone(&self);
        std::thread::spawn(move||{
            // bind separate port for each remote object + client (1 per client)
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
        Ok(())

        // check variable to unbind
        // gracefull shutdown or kill?
    }

    pub fn unbind(&self){
        todo!()
        // switch variable to unbind
    }

    fn handle_connection(&self, mut stream:TcpStream)-> RMIResult<()>{
        let mut len_bytes = [0u8; 4];
        let _ = stream.read_exact(&mut len_bytes)
            .map_err(|e| RMIError::TransportError(e.to_string()))?;
        
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut request_bytes = vec![0u8;len];
        stream.read_exact(&mut request_bytes)
            .map_err(|e| RMIError::TransportError(e.to_string()))?;

        let request: RegistryRequest = serde_cbor::from_slice(&request_bytes)
            .map_err(|e| RMIError::DeserializationError(e.to_string()))?;
        let response = self.handle_request(request);

        let response_bytes = serde_cbor::to_vec(&response)
            .map_err(|e| RMIError::SerializationError(e.to_string()))?;
        let len = response_bytes.len() as u32;

        stream.write_all(&len.to_be_bytes()).map_err(|e| RMIError::TransportError(e.to_string()))?;
        stream.write_all(&response_bytes).map_err(|e| RMIError::TransportError(e.to_string()))?;
        stream.flush().map_err(|e| RMIError::TransportError(e.to_string()))?;

        eprintln!("Response sent.");
        Ok(())
    }

    fn handle_request(&self, req: RegistryRequest) -> RegistryResponse{
        match req{
           RegistryRequest::Lookup { name } => RegistryResponse::Lookup(self.lookup(&name)),
           RegistryRequest::List => RegistryResponse::List(self.list())
        }
    }
}

#[derive(Serialize,Deserialize)]
pub enum RegistryRequest{
    Lookup {name: String},
    List,
}

#[derive(Serialize,Deserialize)]
pub enum RegistryResponse{
    Lookup(RMIResult<RemoteRef>),
    List(RMIResult<Vec<String>>),
}

pub fn get_registry(hostname:&str, port:u16) -> Registry{
    todo!("")// connect to host and port and get remote reference to registry
    // registry = Registry.from(registry.getRegistry("00650072.students.licas.nl",1099))
}

#[cfg(test)]
mod tests{
    use crate::{remote::MockRemoteObject, stub::{Stub,RemoteTrait}};
    use super::*;
    use core::{panic, time};
    use local_ip_address::local_ip;
    use threadpool::ThreadPool;

    #[test]
    fn addr(){
        let reg = Registry::default();
        let port = reg.port;
        let addr = reg.get_addr(port);
        assert_eq!(addr,
            SocketAddr::new(local_ip().expect("Should be able to get ip"), port))
    }


    #[test]
    fn populate_clear(){
        let reg = Arc::new(Mutex::new(Registry::default()));
        let pool = ThreadPool::new(2);
        let jobs = 10;
        let per_thread = 42;

        //REGISTER PHASE
        for thread in 0..jobs{
            let r = Arc::clone(&reg);
            pool.execute( move || {
                for n in 0..per_thread{
                let name = format!("{thread}-{n}");
                let guard = r.lock().unwrap();
                guard.bind(&name, MockRemoteObject::silent());
                drop(guard);
                }
            } );
        }
        
        std::thread::sleep(time::Duration::from_millis(100));
        let num_objects = reg.lock().unwrap().list().unwrap().len();
        eprintln!("Num objects after populating {}",num_objects);
        assert_eq!(num_objects, jobs*per_thread);

        // DEREGISTER PHASE
        for thread in 0..jobs{
            let r = Arc::clone(&reg);
            pool.execute( move || {
                for n in 0..per_thread{
                let guard = r.lock().unwrap();
                let name = format!("{thread}-{n}");
                guard.remove(&name).expect("should still have this process");
                drop(guard);
                }
            } );
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
    fn list_bind_remove(){

        let mut reg = Registry::default();
        let reg = Arc::new(reg);
        let _ = reg.listen();
        let verbose = MockRemoteObject::verbose();
        let silent = MockRemoteObject::silent();
        reg.bind("verbose", verbose);
        reg.bind("silent", silent);

        let remote = reg.lookup("silent").expect("silent should be in");
        eprintln!("{remote:?}") ;
        let remote = reg.lookup("verbose").expect("verbose should be in");
        eprintln!("{remote:?}") ;

        let l = reg.list().expect("two already in");
        eprintln!("{:?}",l);
        reg.remove("verbose").expect("still in");

        let l = reg.list().expect("one still in");
        eprintln!("{:?}",l);
        reg.remove("silent").expect("still in");

        match reg.list(){
            Ok(_)=> panic!("should not have any other objects"),
            Err(RMIError::EmptyRegistry())=> (),
            Err(_)=> panic!("should return EmptyRegistry error")
        };

    }
    #[test]
    fn local_listen(){

        let reg = Registry::default();
        let reg = Arc::new(reg);
        let _ = reg.listen();

        let obj_verbose = MockRemoteObject::verbose();
        let args = vec![42;2];
        let result_expected = obj_verbose.run("locally method_name",args.clone()).expect("Mock object returns the args");
        reg.bind("verbose", obj_verbose);

        let rmt= reg.lookup("verbose").expect("verbose should be in");
        eprintln!("Remote reference: {rmt:?}");

        let stb= Stub::new(rmt);
        eprintln!("Stub: {stb:?}");

        //NEED TO KNOW THE RETURN TYPE
        let res:RMIResult<Vec<u8>> = stb.run_stub(args.clone());
        eprintln!("result: {res:?}\n\n");

        assert_eq!(result_expected,res.clone().unwrap());
        assert_eq!(args,res.clone().unwrap());

        let obj2 = MockRemoteObject::verbose();
        reg.bind("second", obj2);
        let rmt2 = reg.lookup("second").expect("second should be in");
        let stb2 = Stub::new(rmt2);

        let args = "I'm here too!";
        let res: RMIResult<String> = stb.run_stub(args.clone());
        println!("{res:?}")
        

    }
}