pub mod remote;
mod stub;
use remote::RMI_ID;
pub use remote::create_registry;

mod error;
mod transport;
pub use error::RMIError;

extern crate self as rrmi;
pub use remote::{RMIResult, RemoteRef};
pub use stub::{Stub, marshal, unmarshal};
pub use transport::{TcpClient, Transport, receive_data, send_data, utils};
