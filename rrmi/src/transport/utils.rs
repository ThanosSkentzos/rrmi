use if_addrs::Interface;

use crate::transport::tcp::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use crate::{error::RMIError, remote::RMIResult};
use std::str::FromStr;

#[allow(dead_code)]
static START: u16 = 31768;
#[allow(dead_code)]
static END: u16 = 60999;
#[allow(dead_code)]
pub fn get_tcp_socket_linear() -> RMIResult<(TcpListener, u16)> {
    for port in START..END {
        match TcpListener::bind(("0.0.0.0", port)) {
            Ok(l) => return Ok((l, port)),
            _ => {}
        }
    }
    Err(RMIError::TransportError("No available ports".to_string()))
}
pub fn get_tcp_socket_os() -> RMIResult<TcpListener> {
    TcpListener::bind(("0.0.0.0", 0)).map_err(|e| RMIError::TransportError(e.to_string()))
}

pub fn get_addr(hostname: &str, port: u16) -> SocketAddr {
    let ips: Vec<IpAddr> = dns_lookup::lookup_host(hostname)
        .expect("should be able to get own address")
        .collect();
    eprintln!("IPs for {hostname}: {ips:?}");
    if ips.len() == 0 {
        //fail test if not found
        panic!("unable to resolve hostname: {hostname}")
    }
    let mut ip: IpAddr = ips[0];
    if ips
        .iter()
        .any(|ip| ip.to_string().contains("127.0") || ip.to_string().contains("localhost"))
    {
        ip = IpAddr::from(Ipv4Addr::from_str("0.0.0.0").expect("0.0.0.0 should pass"));
        eprintln!("{hostname} is this computer so using {ip:?}");
    }
    eprintln!("using {}:{port} for {hostname}", ip);
    SocketAddr::new(ip, port)
}

pub fn get_local_ips() -> Result<Vec<IpAddr>, ()> {
    let ips = if_addrs::get_if_addrs()
        .map_err(|err| {
            eprintln!("Error getting ips: {err}");
            ()
        })?
        .into_iter()
        .filter(|iface| !iface.is_loopback())
        .map(|iface| iface.ip())
        .collect();
    Ok(ips)
}
#[allow(dead_code)]
pub fn get_local_ifs() -> Result<Vec<Interface>, ()> {
    let ifs = if_addrs::get_if_addrs()
        .map_err(|err| {
            eprintln!("Error getting ips: {err}");
            ()
        })?
        .into_iter()
        .filter(|iface| !iface.is_loopback())
        .collect();
    Ok(ifs)
}
