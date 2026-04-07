pub mod registry;
pub use registry::{RMI_ID, Registry, create_registry, get_registry};

mod remote;
pub use remote::{MockRemoteObject, MockRemoteObjectStub, RMIResult, RemoteObject, RemoteRef};

mod tests;
