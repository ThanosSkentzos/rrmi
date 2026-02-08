use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::error::RMIError;
use crate::remote::{RemoteObject,RMIResult};

pub struct Registry{// a hashmap with all objects
    objects: Arc<Mutex<HashMap<u64,Arc<dyn RemoteObject>>>>,// hashmap and objects should be thread safe
    next_id: Arc<Mutex<u64>>,
}

impl Registry{
    pub fn new() -> Self{
        Registry {
            objects: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn register(&self, object: Arc<dyn RemoteObject>) -> u64{
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id +=1;

        let mut objects = self.objects.lock().unwrap();
        objects.insert(id,object);

        id
    }

    pub fn get(&self, id: u64) -> RMIResult<Arc<dyn RemoteObject>>{
        let objects = self.objects.lock().unwrap();
        objects.get(&id)
            .cloned()
            .ok_or(crate::error::RMIError::ObjectNotFound(id))
    }

    pub fn deregister(&self, id:u64) -> RMIResult<()>{
        let mut objects = self.objects.lock().unwrap();
        objects.remove(&id)
            .ok_or(RMIError::ObjectNotFound(id))?;
        Ok(())
    }

    pub fn count(&self) -> usize{
        let objects = self.objects.lock().unwrap();
        objects.len()
    }

}

