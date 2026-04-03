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

#[remote_object]
impl NumberServer {
    fn new(total_nodes: usize) -> Self {
        let num = 0.into();
        let bar = Barrier::new(total_nodes);
        Self { num, bar }
    }
    #[remote]
    fn get_num(&self) -> u32 {
        self.num.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.num.load(std::sync::atomic::Ordering::SeqCst)
    }
    #[remote]
    fn barrier(&self) -> () {
        let tid = thread::current().id();
        eprintln!("{tid:?} joins the barrier ");
        self.bar.wait();
        eprintln!("{tid:?} escaped!")
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
