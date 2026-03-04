use std::net::TcpListener;
use crate::{error::RMIError, remote::RMIResult};

static START:u16 = 31768;
static END:u16 = 60999;
pub fn find_available_port(taken:&Vec<u16>) -> RMIResult<(TcpListener,u16)> {
    for port in (START..END).filter(|n| !taken.contains(n)) {
         match TcpListener::bind(("0.0.0.0", port)) {
             Ok(l) => return Ok((l,port)),
             _ => {}
         }
    }
    Err(RMIError::TransportError("No available ports".to_string()))
}


#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn get_ports(){
        let mut taken = vec![];
        let (t,p) = find_available_port(&taken)
                .expect("should have available ports");
        let mut a = vec![t];
        let mut port :u16 = 0;
        for _ in 1..100{
            let (t,p) = find_available_port(&taken)
                .expect("should have available ports");
            // eprintln!("{p:?}"); 
            taken.push(p);
            a.push(t);
            port=p;
        }
        eprintln!("{port}")
    }
}