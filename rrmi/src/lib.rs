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

#[cfg(feature = "bench_tcp")]
pub use transport::{
    _send_data_combined, _send_data_ioslice, _send_data_separate, _send_data_separate_flush,
};
