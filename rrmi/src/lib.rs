pub mod remote;
mod stub;
use remote::RMI_ID;
pub use remote::{create_registry, get_registry};

mod error;
mod transport;
pub use error::RMIError;

// need for rrmi_macros
extern crate self as rrmi;
pub use remote::{RMIResult, RemoteRef};
pub use stub::{Stub, marshal, unmarshal};
pub use transport::{TcpClient, TcpStream, Transport, receive_data, send_data, utils};
