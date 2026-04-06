use std::sync::atomic::Ordering::SeqCst;
use std::{
    sync::atomic::{AtomicBool, AtomicU32, AtomicU8},
    thread::{self, sleep},
    time::Duration,
};

use rrmi::{
    create_registry,
    remote::{registry::get_registry, RemoteObject},
};
use rrmi_macros::remote_object;
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
    inbar: AtomicU8,
    total: u8,
    barrier_on: AtomicBool,
    // bar: Barrier,
}

#[remote_object]
impl NumberServer {
    fn new(total: u8) -> Self {
        let num = 0.into();
        // let bar = Barrier::new(total_nodes);
        let inbar = 0.into();
        let barrier_on = AtomicBool::new(false);
        Self {
            num,
            inbar,
            total,
            barrier_on,
        }
    }
    #[remote]
    fn get_num(&self) -> u32 {
        self.num.fetch_add(1, SeqCst);
        self.num.load(SeqCst)
    }
    #[remote]
    fn barrier(&self) -> () {
        // let tid = thread::current().id();
        // eprintln!("{tid:?} joins the barrier ");
        self.barrier_on.store(true, SeqCst);
        self.inbar.fetch_add(1, SeqCst);
        let mut inside = self.inbar.load(SeqCst);
        if inside == self.total {
            self.barrier_on.store(false, SeqCst);
            self.inbar.store(0, SeqCst);
        }
        while inside < self.total {
            inside = self.inbar.load(SeqCst);
            let status = self.barrier_on.load(SeqCst);
            if status == false {
                break;
            }
            sleep(Duration::from_nanos(1));
        }
        // self.bar.wait();
        // eprintln!("{tid:?} escaped!")
    }
}

fn run_thread(stub: &NumberServerStub, char: &str) {
    let times = 10000;
    let _ = stub.barrier();
    for i in 0..times {
        _ = stub.get_num().expect("should be able to get number");
        if i % 1000 == 0 {
            eprint!("{char}");
            _ = stub.barrier();
            if char == "A" {
                eprintln!("|");
            }
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
        run_thread(&stub, "A");
    });
    let reg = get_registry("localhost", port);
    thread::spawn(move || {
        let stub = reg
            .lookup("NumberServer")
            .expect("should be able to get object")
            .into();
        run_thread(&stub, "B");
    });
    run_thread(&stub, "C");
    thread::sleep(Duration::from_millis(100));
    let from_stub = stub.get_num().expect("should be able to get number");
    eprintln!("Number from object: {from_obj} - from stub: {from_stub}");

    // stb.run_stub();
}
