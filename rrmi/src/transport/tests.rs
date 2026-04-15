#[cfg(test)]
mod tests {
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    use crate::utils::{get_local_ifs, get_local_ips, get_tcp_socket_linear, get_tcp_socket_os};
    static TOTAL: usize = 100;
    #[test]
    fn get_own_ips() {
        eprintln!("{:#?}", get_local_ips());
        eprintln!("{:#?}", get_local_ifs());
    }

    #[test]
    fn get_ports() {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should be able to get time");
        let s = Instant::now();
        let (t, _p) = get_tcp_socket_linear().expect("should have available ports");
        let mut a = vec![t];
        for _ in 1..TOTAL {
            let (t, _p) = get_tcp_socket_linear().expect("should have available ports");
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
        let t = get_tcp_socket_os().expect("should have available ports");
        let mut a = vec![t];
        for _ in 1..TOTAL {
            let t = get_tcp_socket_os().expect("should have available ports");
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

#[cfg(test)]
mod tests_transport {
    use std::{
        net::{TcpListener, TcpStream},
        thread,
    };

    use crate::{
        marshal, receive_data, send_data, transport::RMIRequest, unmarshal, utils::get_addr,
    };
    static HOSTNAME_RECV: &str = "0065074.student.liacs.nl";
    static LOCAL_GET_SEND: u16 = 10999;
    static REMOTE_GET_SEND: u16 = 11000;

    #[test]
    #[ignore]
    fn liacs_ips() {
        let hostname = "0.0.0.0";
        get_addr(hostname, 1099);
        let hostname = "localhost";
        get_addr(hostname, 1099);
        let hostname = "0065074.student.liacs.nl";
        get_addr(hostname, 1099);
        let hostname = "0065073.student.liacs.nl";
        get_addr(hostname, 1099);
    }

    #[test]
    fn local_tcp_test() {
        let recv_handle = thread::spawn(|| {
            get_int_struct("localhost", LOCAL_GET_SEND);
        });
        thread::sleep(std::time::Duration::from_millis(100));
        send_int_struct("localhost", LOCAL_GET_SEND);
        recv_handle.join().expect("should be able to join");
    }

    #[test]
    #[ignore]
    fn remote_send() {
        send_int_struct(HOSTNAME_RECV, REMOTE_GET_SEND);
    }
    #[test]
    #[ignore]
    fn remote_recv() {
        get_int_struct(HOSTNAME_RECV, REMOTE_GET_SEND);
    }

    fn get_int_struct(hostname: &str, port: u16) {
        let num: i32 = 1234567890;
        eprintln!("data: {:?}", num);
        let addr = get_addr(hostname, port);
        // let mut stream = TcpStream::connect(addr).unwrap();
        let listener = TcpListener::bind(addr).expect("should be free");
        let (mut stream, _) = listener.accept().expect("should send");
        let bytes = receive_data(&mut stream);
        let num_recv: i32 = unmarshal(&bytes).expect("i32");
        assert_eq!(num_recv, num);

        let req = RMIRequest::default();
        let bytes = receive_data(&mut stream);
        let req_recv: RMIRequest = unmarshal(&bytes).expect("RMIRequest");
        assert_eq!(req_recv, req);
    }

    fn send_int_struct(hostname: &str, port: u16) {
        let addr = get_addr(hostname, port);
        let mut stream = TcpStream::connect(addr).unwrap();
        let int: i32 = 1234567890;
        let int_bytes = marshal(&int).expect("int is serializable");
        eprintln!("data: {:?}", int);
        eprintln!("serialized: {:?}", int_bytes);

        let _ = send_data(int_bytes.clone(), &mut stream);

        let request = RMIRequest::default();
        let request_bytes = marshal(&request).expect("RMIRequest is serializable");
        let _ = send_data(request_bytes, &mut stream);
    }
}
