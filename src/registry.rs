use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{RMIError, RMIResult};
use crate::skeleton::RemoteObject;

pub struct Registry{// a hashmap with all objects
    objects: Arc<RwLock<HashMap<u64,Arc<dyn RemoteObject>>>>,// hashmap and objects should be thread safe
    next_id: Arc<RwLock<u64>>,
}

impl Registry{
    pub fn new() -> Self{
        Registry {
            objects: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn register(&self, object: Arc<dyn RemoteObject>) -> u64{
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id +=1;

        let mut objects = self.objects.write().await;
        objects.insert(id,object);

        id
    }

    pub async fn get(&self, id: u64) -> RMIResult<Arc<dyn RemoteObject>>{
        let objects = self.objects.read().await;
        objects.get(&id)
            .cloned()
            .ok_or(crate::error::RMIError::ObjectNotFound(id))
    }

    pub async fn deregister(&self, id:u64) -> RMIResult<()>{
        let mut objects = self.objects.write().await;
        objects.remove(&id)
            .ok_or(RMIError::ObjectNotFound(id))?;
        Ok(())
    }

    pub async fn count(&self) -> usize{
        let objects = self.objects.write().await;
        objects.len()
    }

}

