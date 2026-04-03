use std::{
    sync::{atomic::AtomicU32, Arc, Barrier},
    thread::{self},
    time::Duration,
};

use rrmi::{
    create_registry,
    remote::{registry::get_registry, RemoteObject},
};
use rrmi_macros::remote_object;
use serde::{Deserialize, Serialize};
struct Calculator;

#[remote_object]
impl Calculator {
    #[remote]
    fn add(&self, a: i32, b: i32, _c: &str) -> i32 {
        a + b
    }
    #[remote]
    fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
    fn sub(&self, a: i32, b: i32) -> i32 {
        a - b
    }
}
struct NumberServer {
    num: AtomicU32,
    bar: Barrier,
}

// #[remote_object]
impl NumberServer {
    fn new(total_nodes: usize) -> Self {
        let num = 0.into();
        let bar = Barrier::new(total_nodes);
        Self { num, bar }
    }
    // #[remote]
    fn get_num(&self) -> u32 {
        self.num.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.num.load(std::sync::atomic::Ordering::SeqCst)
    }
    // #[remote]
    fn barrier(&self) -> () {
        let tid = thread::current().id();
        eprintln!("{tid:?} joins the barrier ");
        self.bar.wait();
        eprintln!("{tid:?} escaped!")
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub enum NumberServerRequest {
    GetNum,
    Barrier,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum NumberServerResponse {
    GetNum(u32),
    Barrier(()),
}

pub struct NumberServerStub {
    remote: ::rrmi::RemoteRef,
}
impl From<::rrmi::Stub> for NumberServerStub {
    fn from(stub: ::rrmi::Stub) -> Self {
        NumberServerStub {
            remote: stub.remote,
        }
    }
}
impl NumberServerStub {
    pub fn get_num(&self) -> ::rrmi::RMIResult<u32> {
        use ::rrmi::Transport;
        let transport_client = ::rrmi::TcpClient::new(self.remote.addr);
        let req = NumberServerRequest::GetNum {};
        let resp: NumberServerResponse = transport_client.send(req)?;
        match resp {
            NumberServerResponse::GetNum(res) => Ok(res),
            _ => Err(::rrmi::RMIError::TransportError(
                "Wrong response".to_string(),
            )),
        }
    }
    pub fn barrier(&self) -> ::rrmi::RMIResult<()> {
        use ::rrmi::Transport;
        let transport_client = ::rrmi::TcpClient::new(self.remote.addr);
        let req = NumberServerRequest::Barrier {};
        // eprintln!("sending Request:{req:?} to {:?}", self.remote.addr);
        let resp: NumberServerResponse = transport_client.send(req)?;
        match resp {
            NumberServerResponse::Barrier(res) => Ok(res),
            _ => Err(::rrmi::RMIError::TransportError(
                "Wrong response".to_string(),
            )),
        }
    }
}
impl RemoteObject for NumberServer {
    fn run(&self, stream: &mut ::rrmi::TcpStream) -> ::rrmi::RMIResult<()> {
        let s = Arc::new(self);
        s.handle_connection_gen(stream)
    }
}

impl NumberServer {
    fn handle_connection_gen(
        self: Arc<&Self>,
        stream: &mut ::rrmi::TcpStream,
    ) -> ::rrmi::RMIResult<()> {
        let request_bytes = ::rrmi::receive_data(stream);
        let request: NumberServerRequest = ::rrmi::unmarshal(&request_bytes)?;
        let response: NumberServerResponse = self.handle_request_gen(request);
        let response_bytes = ::rrmi::marshal(&response)?;
        ::rrmi::send_data(response_bytes, stream)
    }
    fn handle_request_gen(self: Arc<&Self>, req: NumberServerRequest) -> NumberServerResponse {
        match req {
            NumberServerRequest::GetNum => NumberServerResponse::GetNum(self.get_num()),
            NumberServerRequest::Barrier => NumberServerResponse::Barrier(self.barrier()),
        }
    }
}

fn run_thread(stub: &NumberServerStub) {
    let times = 10000;
    for i in 0..times {
        _ = stub.get_num().expect("should be able to get number");
        if i % 1000 == 0 {
            _ = stub.barrier();
        }
    }
}
fn main() {
    let cal = Calculator;
    let (a, b, c) = (1, 2, "test");
    cal.add(a, b, "test");
    cal.multiply(a, b);
    cal.sub(a, b);

    let port = 1099;
    let registry = create_registry(port);
    let name = "calc";
    registry.bind(name, cal);
    let reg = get_registry("localhost", port);
    let calc: CalculatorStub = reg
        .lookup(name)
        .expect("Should be able to get object")
        .into();
    let _res = calc.add(a, b, c);

    let numserver = NumberServer::new(3);
    let from_obj = numserver.get_num();
    registry.bind("NumberServer", numserver);

    let stub = reg
        .lookup("NumberServer")
        .expect("should be able to get object")
        .into();
    thread::spawn(move || {
        let stub = reg
            .lookup("NumberServer")
            .expect("should be able to get object")
            .into();
        run_thread(&stub);
    });
    let reg = get_registry("localhost", port);
    thread::spawn(move || {
        let stub = reg
            .lookup("NumberServer")
            .expect("should be able to get object")
            .into();
        run_thread(&stub);
    });
    run_thread(&stub);
    thread::sleep(Duration::from_millis(100));
    let from_stub = stub.get_num().expect("should be able to get number");
    eprintln!("Number from object: {from_obj} - from stub: {from_stub}");

    // stb.run_stub();
}
