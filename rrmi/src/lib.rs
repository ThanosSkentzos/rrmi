mod stub;

pub mod remote;
use remote::RMI_ID;
pub use remote::create_registry;

mod transport;
use transport::TcpClient;
mod error;
pub use error::RMIError;

extern crate self as rrmi;
pub use remote::RMIResult;
pub use stub::{marshal, unmarshal};
pub use transport::utils;
pub use transport::{receive_data, send_data};
