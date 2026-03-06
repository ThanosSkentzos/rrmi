use crate::{error::RMIError, remote::RMIResult};
use crate::transport::tcp::TcpListener;
pub static START: u16 = 31768;
static END: u16 = 60999;
pub fn find_available_port_mine() -> RMIResult<(TcpListener, u16)> {
    for port in START..END {
        match TcpListener::bind(("0.0.0.0", port)) {
            Ok(l) => return Ok((l, port)),
            _ => {}
        }
    }
    Err(RMIError::TransportError("No available ports".to_string()))
}
pub fn find_available_port_os() -> RMIResult<(TcpListener)> {
    TcpListener::bind(("0.0.0.0", 0)).map_err(|e| RMIError::TransportError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    use super::*;
    static TOTAL: usize = 1000;
    #[test]
    fn get_ports_mine() {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should be able to get time");
        let s = Instant::now();
        let (t, p) = find_available_port_mine().expect("should have available ports");
        let mut a = vec![t];
        for _ in 1..TOTAL {
            let (t, p) = find_available_port_mine().expect("should have available ports");
            // eprintln!("{p:?}");
            a.push(t);
        }
        let end = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should be able to get time");
        let e = Instant::now();
        eprintln!("time taken SystemTime: {:?}", end - start);
        eprintln!("time taken Instant: {:?}", e - s);
    }
    #[test]
    fn get_ports_os() {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should be able to get time");
        let s = Instant::now();
        let t = find_available_port_os().expect("should have available ports");
        let mut a = vec![t];
        for _ in 1..TOTAL {
            let t = find_available_port_os().expect("should have available ports");
            // eprintln!("{t:?}");
            a.push(t);
        }
        let end = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should be able to get time");
        let e = Instant::now();
        eprintln!("time taken SystemTime: {:?}", end - start);
        eprintln!("time taken Instant: {:?}", e - s);
    }
}
