
#[cfg(test)]
mod tests {
    use std::time::{Instant, SystemTime, UNIX_EPOCH};
    use super::super::utils::{get_local_ips,get_local_ifs,get_tcp_socket_manual,get_tcp_socket};
    static TOTAL: usize = 100;
    #[test]
    fn get_own_ips() {
        eprintln!("{:#?}", get_local_ips());
        eprintln!("{:#?}", get_local_ifs());
    }

    #[test]
    fn get_ports_mine() {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should be able to get time");
        let s = Instant::now();
        let (t, _p) = get_tcp_socket_manual().expect("should have available ports");
        let mut a = vec![t];
        for _ in 1..TOTAL {
            let (t, _p) = get_tcp_socket_manual().expect("should have available ports");
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
        let t = get_tcp_socket().expect("should have available ports");
        let mut a = vec![t];
        for _ in 1..TOTAL {
            let t = get_tcp_socket().expect("should have available ports");
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