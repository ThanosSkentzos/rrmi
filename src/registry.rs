use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::error::RMIError;
use crate::remote::{RMIResult, RemoteObject, RemoteRef};
pub type RMI_ID = u16;

pub struct Registry{// a hashmap with all objects
    port: u16,
    objects: Arc<Mutex<HashMap<RMI_ID,Arc<dyn RemoteObject>>>>,// hashmap and objects should be thread safe
    names: Arc<Mutex<HashMap<String,RMI_ID>>>,
    next_id: Arc<Mutex<RMI_ID>>,
}

impl Registry{
    pub fn new() -> Self{
        Registry {
            port: 1099,
            objects: Arc::new(Mutex::new(HashMap::new())),
            names: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    fn get_addr(&self)-> SocketAddr{
        // this will be slower than just saving it
        let ip = local_ip_address::local_ip().expect("Should be able to get local ip");
        SocketAddr::new(ip, self.port)
    }

    pub fn register(&self,name:String,object: Arc<dyn RemoteObject>) -> RMI_ID{
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id +=1;
        self.objects.lock().unwrap().insert(id, object);
        self.names.lock().unwrap().insert(name, id);
        id
    }

    pub fn deregister(&self, name:String) -> RMIResult<()>{
        let mut names = self.names.lock().unwrap();
        let id = names.get(&name).cloned().ok_or(RMIError::NameNotFound(name.clone()))?;
        names.remove(&name);

        let mut objects = self.objects.lock().unwrap();
        objects.remove(&id)
            .ok_or(RMIError::ObjectNotFound(id))?;
        Ok(())
    }

    pub fn get(&self, id: RMI_ID) -> RMIResult<Arc<dyn RemoteObject>>{
        // ask for remote object according to id
        let objects = self.objects.lock().unwrap();
        objects.get(&id)
            .cloned()
            .ok_or(RMIError::ObjectNotFound(id))
    }

    pub fn lookup(&self,name:&str) -> RMIResult<RemoteRef>{
        // ask for remote ref according to name
        let names = self.names.lock().unwrap();
        names.get(name)
            .ok_or(RMIError::NameNotFound(name.to_string()))
            .map(|&id| RemoteRef { addr: self.get_addr(), id })
    }


    pub fn list(&self) -> RMIResult<Vec<String>>{
        let names: Vec<String> = self.names.lock().unwrap().keys().cloned().collect();
        match names.len(){
            0 => RMIResult::Err(RMIError::EmptyRegistry()),
            _ => RMIResult::Ok(names)
        }
    }

    pub fn bind(&self){
        todo!()
        // try TcpListener at 0.0.0.0:port otherwise error
        // spawn some workers to catch requests and run handle_request
        // check variable to unbind
        // gracefull shutdown or kill?
    }

    pub fn unbind(&self){
        todo!()
        // switch variable to unbind
    }

    pub fn handle_request(&mut self, req: RegistryRequest) -> RegistryResponse{
        match req{
           RegistryRequest::Lookup { name } => RegistryResponse::Lookup(self.lookup(&name)),
           RegistryRequest::List => RegistryResponse::List(self.list())
        }
    }
}

pub enum RegistryRequest{
    Lookup {name: String},
    List,
}

pub enum RegistryResponse{
    Lookup(RMIResult<RemoteRef>),
    List(RMIResult<Vec<String>>),
}

pub fn getRegistry(hostname:&str, port:u16) -> Registry{
    todo!("")// connect to host and port and get remote reference to registry
    // registry = Registry.from(registry.getRegistry("00650072.students.licas.nl",1099))
}

#[cfg(test)]
mod tests{
    use core::time;
    use super::*;
    use local_ip_address::local_ip;
    use threadpool::ThreadPool;
    #[test]
    fn addr(){
        let reg = Registry::new();
        let port = reg.port;
        let addr = reg.get_addr();
        assert_eq!(addr,
            SocketAddr::new(local_ip().expect("Should be able to get ip"), port))
    }


    struct TestObject{}
    impl RemoteObject for TestObject{
        fn run(&self, method_name: &str, args: Vec<u8>) -> RMIResult<Vec<u8>> {
            RMIResult::Ok(Vec::new())
        }
    }
    #[test]
    fn populate_clear(){
        let reg = Arc::new(Mutex::new(Registry::new()));
        let pool = ThreadPool::new(4);
        let jobs = 10;
        let per_thread = 42;

        //REGISTER PHASE
        for thread in 0..jobs{
            let r = Arc::clone(&reg);
            pool.execute( move || {
                for n in 0..per_thread{
                let object = Arc::new(TestObject{});
                let name = format!("{thread}-{n}");
                let guard = r.lock().unwrap();
                guard.register(name, object);
                drop(guard);
                }
            } );
        }
        std::thread::sleep(time::Duration::from_millis(100));
        let num_objects = reg.lock().unwrap().list().unwrap().len();
        println!("Num objects after populating {}",num_objects);
        assert_eq!(num_objects, jobs*per_thread);

        // DEREGISTER PHASE
        for thread in 0..jobs{
            let r = Arc::clone(&reg);
            pool.execute( move || {
                for n in 0..per_thread{
                let guard = r.lock().unwrap();
                let name = format!("{thread}-{n}");
                guard.deregister(name).expect("should still have this process");
                drop(guard);
                }
            } );
        }

        std::thread::sleep(time::Duration::from_millis(100));
        let names = reg.lock().unwrap().list();
         
        assert_eq!(names.err(), Option::Some(RMIError::EmptyRegistry()));
    }
}