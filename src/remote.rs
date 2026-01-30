use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Clone,Serialize,Deserialize)]
pub struct RemoteRef{//should point to RemoteObject on the server side
    pub addr: SocketAddr,   // 127.0.0.1:8080 for example
    pub id: u64,            // just a num for identity
}