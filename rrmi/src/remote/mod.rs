pub mod registry;
pub use registry::{RMI_ID, Registry, create_registry};
use rrmi_macros::remote_object;

use crate::TcpStream;
use crate::error::RMIError;
use crate::stub::{Deserialize, Serialize};
use crate::transport::{IpAddr, SocketAddr};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RemoteRef {
    //should point to RemoteObject on the server side
    pub addr: SocketAddr, // 127.0.0.1:8080 for example
    pub id: RMI_ID,       // just a num for identity
}
impl RemoteRef {
    pub fn new(addr: SocketAddr, id: RMI_ID) -> Self {
        RemoteRef { addr, id }
    }
    pub fn example() -> Self {
        let addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 1099);
        RemoteRef { addr, id: 1 }
    }
}

pub trait RemoteObject: Send + Sync {
    // fn listen(self: &Arc<Self>) -> RMIResult<u16>;
    // CANNOT USE AS DYNAMIC WITH &Arc ref

    fn handle_connection(&mut self, stream: &mut TcpStream) -> RMIResult<()>;

    // fn handle_request<ObjReq, ObjRes>(&self, req: ObjReq) -> ObjRes;
    //CANNOT USE AS DYNAMIC WITH generic types
}

pub type RMIResult<T> = Result<T, RMIError>;

pub struct MockRemoteObject {
    verbose: bool,
}

#[remote_object]
impl MockRemoteObject {
    pub fn new() -> MockRemoteObject {
        MockRemoteObject { verbose: true }
    }

    pub fn verbose() -> MockRemoteObject {
        MockRemoteObject { verbose: true }
    }
    pub fn silent() -> MockRemoteObject {
        MockRemoteObject { verbose: false }
    }

    #[remote]
    fn run(&self, method_name: &str, args: Vec<u8>) -> Vec<u8> {
        if self.verbose {
            eprintln!("Remote got {method_name} and vec: {args:?}");
            // let result: Vec<u8> =
            // unmarshal(&args)
            // .expect("should be able to deserialize");
            // eprintln!("After deserializing: {result:?}");
        }
        args
    }
}
