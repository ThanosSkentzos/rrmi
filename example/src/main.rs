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

//=============================TRACING============================

#[cfg(debug_assertions)]
use tracing::instrument;
#[cfg(debug_assertions)]
use tracing_chrome::ChromeLayerBuilder;
#[cfg(debug_assertions)]
#[allow(unused)]
use tracing_subscriber::{prelude::*, registry::Registry};

#[allow(unused)]
#[cfg_attr(debug_assertions, derive(Debug))]
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
    #[cfg_attr(debug_assertions, instrument)]
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
    #[cfg_attr(debug_assertions, instrument)]
    fn get_num_atomic(&self) -> u32 {
        self.num.fetch_add(1, SeqCst);
        self.num.load(SeqCst)
    }

    #[remote]
    #[cfg_attr(debug_assertions, instrument)]
    fn get_num_mutex(&self) -> u32 {
        let mut num2 = self.num2.lock().unwrap();
        *num2 += 1;
        *num2
    }
    #[remote]
    #[cfg_attr(debug_assertions, instrument)]
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

#[cfg_attr(debug_assertions, instrument)]
fn run_thread(stub: &NumberServerStub, char: &str) {
    let times = 33333;
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
#[cfg_attr(debug_assertions, instrument)]
fn run_thread_a(stub: &NumberServerStub) {
    run_thread(stub, "A");
}
#[cfg_attr(debug_assertions, instrument)]
fn run_thread_b(stub: &NumberServerStub) {
    run_thread(stub, "B");
}
#[cfg_attr(debug_assertions, instrument)]
fn run_thread_c(stub: &NumberServerStub) {
    run_thread(stub, "C");
}

fn main() {
    #[cfg(debug_assertions)]
    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    #[cfg(debug_assertions)]
    tracing_subscriber::registry().with(chrome_layer).init();

    let port = 1099;
    eprintln!("Creating Registry");
    let registry = create_registry(port);
    eprintln!("Getting RegistryStub");
    let reg = get_registry("localhost", port);

    let numserver = NumberServer::new(3);
    eprintln!("Binding NumberServer");
    registry.bind("NumberServer", numserver);

    let t = Instant::now();

    eprintln!("Making thread A");
    let ahandle = thread::Builder::new()
        .name("Thread A".to_string())
        .spawn(move || {
            let reg = get_registry("localhost", port);
            let stub = reg
                .lookup("NumberServer")
                .expect("stub lookup failed")
                .into();
            run_thread_a(&stub);
        })
        .expect("Could not spawn thread A");
    eprintln!("Making thread B");
    let bhandle = thread::Builder::new()
        .name("Thread B".to_string())
        .spawn(move || {
            let reg = get_registry("localhost", port);
            let stub = reg
                .lookup("NumberServer")
                .expect("stub lookup failed")
                .into();
            run_thread_b(&stub);
        })
        .expect("Could not spawn thread A");
    eprintln!("Making thread C");
    let chandle = thread::Builder::new()
        .name("Thread C".to_string())
        .spawn(move || {
            let reg = get_registry("localhost", port);
            let stub = reg
                .lookup("NumberServer")
                .expect("stub lookup failed")
                .into();
            run_thread_c(&stub);
        })
        .expect("Could not spawn thread C");
    let stub: NumberServerStub = reg
        .lookup("NumberServer")
        .expect("stub lookup failed")
        .into();
    ahandle.join().expect("thread did not join");
    bhandle.join().expect("thread did not join");
    chandle.join().expect("thread did not join");

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
