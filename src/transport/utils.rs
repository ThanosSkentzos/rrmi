use crate::{error::RMIError, remote::RMIResult};
use crate::transport::tcp::{TcpListener,IpAddr,Ipv4Addr,SocketAddr};
use std::str::FromStr;

static START: u16 = 31768;
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

pub fn get_local_addr(port:u16) -> SocketAddr {
    let hostname = "localhost";
    let ips: Vec<IpAddr> = dns_lookup::lookup_host(hostname).unwrap().collect();
    eprintln!("{hostname} ips: {ips:?}");
    SocketAddr::new(ips[0], port) // TODO for now use 1st entry
}

pub fn get_server_addr(hostname: &str,port:u16) -> SocketAddr {
    let ips: Vec<IpAddr> = dns_lookup::lookup_host(hostname).unwrap().collect();
    eprintln!("{hostname} ips: {ips:?}");
    if ips.len() == 0 {
        //fail test if not found
        panic!("unable to resolve hostname: {hostname}")
    }
    let mut ip: IpAddr = ips[0];
    if ips.iter().any(|ip| ip.to_string().contains("127.0") || ip.to_string().contains("localhost")) {
        ip = IpAddr::from(Ipv4Addr::from_str("0.0.0.0").expect("0.0.0.0 should pass"));
        eprintln!("{hostname} is this computer so using {ip:?}");
    }
    eprintln!("using {}:{port} for {hostname}", ip);
    SocketAddr::new(ip, port)
}

#[cfg(test)]
mod tests {
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    use super::*;
    static TOTAL: usize = 100;
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
