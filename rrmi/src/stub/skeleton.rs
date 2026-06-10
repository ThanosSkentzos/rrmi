use std::fmt::Debug;
use std::sync::Arc;

#[cfg(feature = "tracing")]
use tracing::instrument;
#[cfg(feature = "tracing")]
use tracing::{Level, span};

use crate::TransportServer;
use crate::remote::{RMIResult, RemoteObject};

pub struct Skeleton {
    object: Arc<dyn RemoteObject>, // Arc because eventually we to listen from several ports
}

impl Skeleton {
    pub fn new(object: Arc<dyn RemoteObject>) -> Self {
        Skeleton { object }
    }

    #[cfg_attr(feature = "tracing", instrument)]
    pub fn listen<Server: TransportServer + Send + Sync>(&self) -> RMIResult<u16> {
        let obj_clone = Arc::clone(&self.object);
        let object_name = obj_clone.name();

        let transport_server = Server::new(obj_clone);
        // let transport_server = TcpServer::new(obj_clone);
        let addr = transport_server.get_address();
        let port = addr.port();
        eprintln!("{object_name} uses address: {addr}");

        let name = format!("Skeleton{object_name}:{port}");
        let _thread_handle_obj = std::thread::Builder::new().name(name).spawn(move || {
            #[cfg(feature = "tracing")]
            let span = span!(Level::TRACE, "listen");
            #[cfg(feature = "tracing")]
            let _enter = span.enter();
            transport_server.listen();
        });
        Ok(addr.port())
    }
}

impl Debug for Skeleton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Skeleton[{:?}]", self.object.name())
    }
}
//#TODO tests
