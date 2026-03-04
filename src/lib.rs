mod stub;

pub mod remote;
use remote::RMI_ID;
pub use remote::Registry;

mod transport;
use transport::TcpClient;

mod error;