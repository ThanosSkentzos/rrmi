#[cfg(test)]
mod tests {
    use crate::remote::Registry;
    use crate::remote::registry::get_registry;
    use crate::transport::{SocketAddr, TcpListener, TcpStream};
    use crate::utils::get_local_ips;
    use crate::{RMIError, create_registry};
    use crate::{
        receive_data,
        remote::{MockRemoteObject, MockRemoteObjectStub},
        send_data,
        stub::{Stub, marshal, unmarshal},
    };
    use core::{panic, time};
    use std::sync::{Arc, Mutex};
    #[allow(unused_imports)]
    use std::{io::Read, thread, time::Duration};
    use threadpool::ThreadPool;

    static POPUL_PORT: u16 = 10996;
    static BIND_PORT: u16 = 10997;
    static LOCAL_PORT: u16 = 10998;
    static REMOTE_TEST_PORT: u16 = 12345;
    static REMOTE_TEST_SYNC_PORT: u16 = 54321;
    static REMOTE_HOST: &str = "0065074.student.liacs.nl";

    #[test]
    fn addr() {
        let reg = Registry::default();
        let port = reg.port;
        let addr = reg.get_addr(port);
        let ip = get_local_ips().expect("Should be able to get ips")[0];
        assert_eq!(addr, SocketAddr::new(ip, port))
    }

    #[test]
    fn populate_clear() {
        let reg = create_registry(POPUL_PORT);
        let reg = Arc::new(Mutex::new(reg));
        let pool = ThreadPool::new(2);
        let jobs = 10;
        let per_thread = 42;

        //REGISTER PHASE
        for thread in 0..jobs {
            let r = Arc::clone(&reg);
            pool.execute(move || {
                for n in 0..per_thread {
                    let name = format!("{thread}-{n}");
                    let guard = r.lock().unwrap();
                    guard.bind(&name, MockRemoteObject::silent());
                    drop(guard);
                }
            });
        }

        std::thread::sleep(time::Duration::from_millis(100));
        let num_objects = reg.lock().unwrap().list().unwrap().len();
        eprintln!("Num objects after populating {}", num_objects);
        assert_eq!(num_objects, jobs * per_thread);

        // DEREGISTER PHASE
        for thread in 0..jobs {
            let r = Arc::clone(&reg);
            pool.execute(move || {
                for n in 0..per_thread {
                    let guard = r.lock().unwrap();
                    let name = format!("{thread}-{n}");
                    guard.remove(&name).expect("should still have this process");
                    drop(guard);
                }
            });
        }

        std::thread::sleep(time::Duration::from_millis(100));
        let names = reg.lock().unwrap().list();

        match names {
            Result::Err(RMIError::EmptyRegistry()) => (),
            _ => panic!(),
        }
        // assert_eq!(names.err(), Option::Some(RMIError::EmptyRegistry()));
    }

    #[test]
    fn bind_lookup_list_remove() {
        let reg = create_registry(BIND_PORT);
        let rmt_reg = get_registry("localhost", BIND_PORT);

        let verbose = MockRemoteObject::verbose();
        let silent = MockRemoteObject::silent();
        reg.bind("verbose", verbose);
        reg.bind("silent", silent);

        let _remote = reg.lookup_log("silent").expect("silent should be in");
        let _remote = reg.lookup_log("verbose").expect("verbose should be in");

        let l = reg.list().expect("two already in");
        let l_rmt = rmt_reg.list().expect("same");
        eprintln!("local: {:?} vs remote: {:?}", l, l_rmt);
        reg.remove_log("verbose").expect("still in");

        let l = reg.list().expect("one still in");
        let l_rmt = rmt_reg.list().expect("same");
        eprintln!("local: {:?} vs remote: {:?}", l, l_rmt);
        reg.remove_log("silent").expect("still in");

        match reg.list() {
            Ok(_) => panic!("should not have any other objects"),
            Err(RMIError::EmptyRegistry()) => (),
            Err(_) => panic!("should return EmptyRegistry error"),
        };
    }

    #[test]
    fn local_skel_stub() {
        let obj_verbose = MockRemoteObject::verbose();
        let args = vec![42; 2];
        let res_expected = args.clone();
        eprintln!("args: {args:?}");

        eprintln!("reg preparation");
        let reg = create_registry(LOCAL_PORT);
        reg.bind("verbose", obj_verbose);
        let rmt_reg = get_registry("localhost", LOCAL_PORT);
        let stb = rmt_reg.lookup("verbose").expect("verbose should be in");
        eprintln!("Stub: {stb:?} will turn into MockRemoteObjectStub");
        let stub = MockRemoteObjectStub::from(stb);
        let res = stub
            .run("first test", args)
            .expect("MockObject returns the args");
        //NEED TO KNOW THE RETURN TYPE
        // let res: RMIResult<Vec<u8>> = stb.run_stub(args.clone());
        assert_eq!(res_expected, res.clone());
        eprintln!("result: {res:?} matched expected\n\n");

        let obj2 = MockRemoteObject::verbose();
        let args2 = "I'm here too!";
        let sargs2 = marshal(&args2).expect("should be able to serialize");
        // let resp2 = obj2
        //     .run("locally method_name", sargs2)
        //     .expect("Mock object returns the args");
        reg.bind("second", obj2);
        let rmt2 = reg.lookup_log("second").expect("second should be in");
        let stb2 = Stub::new(rmt2);
        let stub2: MockRemoteObjectStub = stb2.into();
        #[allow(noop_method_call)]
        let res2 = stub2
            .run("mothod_name", sargs2.clone())
            .expect("Mock object returns the args");
        let res2_expected: String = unmarshal(&res2).expect("should be able to deserialize");
        eprintln!("result: {res2:?} matched expected\n\n");
        assert_eq!(args2, res2_expected);
    }

    #[test]
    #[ignore]
    fn remote_skel() {
        // assume it runs on 0065074.student.liacs.nl
        let reg = create_registry(REMOTE_TEST_PORT);
        let obj_verbose = MockRemoteObject::verbose();
        reg.bind("verbose", obj_verbose);
        assert_eq!(block_receiver(REMOTE_TEST_SYNC_PORT), vec![0])
    }

    #[test]
    #[ignore]
    fn remote_stub() {
        // runs after remote_listen on 00650??.student.liacs.nl
        let reg = get_registry(REMOTE_HOST, REMOTE_TEST_PORT);
        let stub: MockRemoteObjectStub = reg.lookup("verbose").expect("should work").into();
        let res = stub.run("send the data", vec![42; 2]);
        println!("{res:?}");
        let resp = res.expect("MockObject sends the args back over the network");
        println!("{resp:?}");
        block_sender(REMOTE_HOST, REMOTE_TEST_SYNC_PORT);
    }

    fn ensure_connect(socket: &str) -> TcpStream {
        let stream: TcpStream;
        loop {
            let s = TcpStream::connect(socket);
            match s {
                Ok(strm) => {
                    stream = strm;
                    break;
                }
                Err(_e) => continue,
            }
        }
        stream
    }

    fn block_receiver(port: u16) -> Vec<u8> {
        let l = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("should be able to get port");
        let (mut stream, _) = l.accept().expect("send message from skel");
        receive_data(&mut stream)
    }

    fn block_sender(host: &str, port: u16) {
        let socket = format!("{host}:{}", port);
        let mut stream = ensure_connect(&socket);
        let data_serial = vec![0];
        let _ = send_data(data_serial, &mut stream);
    }
}
