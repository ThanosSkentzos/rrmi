use std::sync::atomic::Ordering::SeqCst;
use std::sync::Barrier;
use std::time::Instant;
use std::{
    sync::atomic::{AtomicBool, AtomicU32, AtomicU8},
    sync::Mutex,
    thread,
    time::Duration,
};

use rrmi::{create_registry, get_registry, remote::RemoteObject};
use rrmi_macros::remote_object;
use thousands::Separable;

#[allow(unused)]
struct NumberServer {
    num: AtomicU32,
    num2: Mutex<u32>,
    inbar: AtomicU8,
    total: u8,
    barrier_on: AtomicBool,
    bar: Barrier,
}

#[remote_object]
impl NumberServer {
    fn new(total: u8) -> Self {
        let num = 0.into();
        let num2 = Mutex::new(0);
        let bar = Barrier::new(total as usize);
        let inbar = 0.into();
        let barrier_on = AtomicBool::new(false);
        Self {
            num,
            num2,
            inbar,
            total,
            barrier_on,
            bar,
        }
    }
    #[remote]
    fn get_num_atomic(&self) -> u32 {
        self.num.fetch_add(1, SeqCst);
        self.num.load(SeqCst)
    }

    #[remote]
    fn get_num_mutex(&self) -> u32 {
        let mut num2 = self.num2.lock().unwrap();
        *num2 += 1;
        *num2
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
            thread::sleep(Duration::from_nanos(1));
        }
        // self.bar.wait();
        // eprintln!("{tid:?} escaped!")
    }
}

fn run_thread(stub: &NumberServerStub, char: &str) {
    let times = 333333;
    let barrier_count = 10000;
    let _ = stub.barrier();
    for i in 0..times {
        _ = stub.get_num_atomic().expect("should be able to get number");
        if i % barrier_count == 0 {
            _ = stub.get_num_mutex().expect("same");
            eprint!("{char}");
            _ = stub.barrier();
            if char == "A" {
                eprintln!("|");
            }
        }
    }
}

fn main() {
    // let cal = Calculator;
    // let (a, b, c) = (1, 2, "test");
    // cal.add(a, b, "test");
    // cal.multiply(a, b);
    // cal.sub(a, b);

    let port = 1099;
    eprintln!("Creating Registry");
    let registry = create_registry(port);
    eprintln!("Getting RegistryStub");
    let reg = get_registry("localhost", port);

    // let name = "calc";
    // registry.bind(name, cal);
    // let calc: CalculatorStub = reg
    //     .lookup(name)
    //     .expect("Should be able to get object")
    //     .into();
    // let _res = calc.add(a, b, c);

    let numserver = NumberServer::new(3);
    eprintln!("Binding NumberServer");
    registry.bind("NumberServer", numserver);

    let t = Instant::now();

    let stub = reg
        .lookup("NumberServer")
        .expect("stub lookup failed")
        .into();
    eprintln!("Making thread A");
    let ahandle = thread::Builder::new()
        .name("A".to_string())
        .spawn(move || {
            let reg = get_registry("localhost", port);
            let stub = reg
                .lookup("NumberServer")
                .expect("stub lookup failed")
                .into();
            run_thread(&stub, "A");
        })
        .expect("Could not spawn thread A");
    eprintln!("Making thread B");
    let bhandle = thread::Builder::new()
        .name("B".to_string())
        .spawn(move || {
            let reg = get_registry("localhost", port);
            let stub = reg
                .lookup("NumberServer")
                .expect("stub lookup failed")
                .into();
            run_thread(&stub, "B");
        })
        .expect("Could not spawn thread B");
    run_thread(&stub, "C");
    thread::sleep(Duration::from_millis(100));
    ahandle.join().expect("thread did not join");
    bhandle.join().expect("thread did not join");
    let atomic = stub.get_num_atomic().expect("stub get_num failed");
    let mutex = stub.get_num_mutex().expect("stub get_num failed");
    let time = t.elapsed();

    eprintln!(
        "Total atomic: {} & mutex: {}",
        atomic.separate_with_underscores(),
        mutex.separate_with_underscores()
    );
    eprintln!("Total time: {:?}", t.elapsed());
    eprintln!("Average: {:?}", time / atomic);

    // stb.run_stub();
}
