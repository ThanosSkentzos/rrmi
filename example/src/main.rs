use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Barrier, Condvar};
use std::thread::current;
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
    count: AtomicU8,
    total: u8,
    barrier_on: AtomicBool,
    bar: Barrier,
    count_mut: Mutex<u8>,
    barrier_num: Mutex<usize>,
    condvar: Condvar,
}

#[remote_object]
impl NumberServer {
    #[cfg_attr(debug_assertions, instrument)]
    fn new(total: u8) -> Self {
        let num = 0.into();
        let num2 = Mutex::new(0);
        let count = 0.into();
        let barrier_on = AtomicBool::new(false);
        let bar = Barrier::new(total as usize);
        let count_mut = Mutex::new(0);
        let barrier_num = Mutex::new(0);
        let condvar = Condvar::new();
        Self {
            num,
            num2,
            count,
            total,
            barrier_on,
            bar,
            count_mut,
            barrier_num,
            condvar,
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
    fn barrier_atomic(&self) -> () {
        self.barrier_on.store(true, SeqCst);
        self.count.fetch_add(1, SeqCst);
        let mut inside = self.count.load(SeqCst);
        if inside == self.total {
            self.barrier_on.store(false, SeqCst);
            self.count.store(0, SeqCst);
        }
        while inside < self.total {
            inside = self.count.load(SeqCst);
            let status = self.barrier_on.load(SeqCst);
            if status == false {
                break;
            }
            thread::sleep(Duration::from_micros(10));
        }
    }
    #[remote]
    #[cfg_attr(debug_assertions, instrument)]
    fn barrier_bar(&self) -> () {
        self.bar.wait();
    }

    #[remote]
    #[cfg_attr(debug_assertions, instrument)]
    fn barrier_mutex(&self) -> () {
        let barrier_num = self.barrier_num.lock().unwrap();
        let current_num = *barrier_num;
        let mut count = self.count_mut.lock().unwrap();
        *count += 1;
        let current_count = *count;
        drop(count);
        drop(barrier_num);
        if current_count < self.total {
            let _res = self
                .condvar
                .wait_while(self.barrier_num.lock().unwrap(), |num| current_num == *num);
        } else {
            *self.count_mut.lock().unwrap() = 0;
            *self.barrier_num.lock().unwrap() += 1;
            self.condvar.notify_all();
        }
    }
}

#[cfg_attr(debug_assertions, instrument)]
fn run_thread(stub: &NumberServerStub, char: &str) {
    let times = 33;
    let barrier_count = 10;
    let _ = stub.barrier_mutex();
    for i in 0..times {
        _ = stub.get_num_atomic().expect("should be able to get number");
        if i % barrier_count == 0 {
            _ = stub.get_num_mutex().expect("same");
            eprint!("{char}");
            _ = stub.barrier_mutex();
            if char == "0" {
                eprintln!("|");
            }
        }
    }
}

#[cfg_attr(debug_assertions, instrument)]
fn run_threads(n: u8, port: u16) {
    let mut handles = vec![];
    for i in 0..n {
        let handle = thread::Builder::new()
            .name(format!("Stub{i}"))
            .spawn(move || {
                let reg = get_registry("localhost", port);
                let stub = reg
                    .lookup("NumberServer")
                    .expect("stub lookup failed")
                    .into();
                run_thread(&stub, i.to_string().as_ref());
            })
            .expect("Could not spawn thread.");
        handles.push(handle);
    }
    for handle in handles {
        handle.join().expect("Could not join handle");
    }
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

    let numthreads = 8;
    let numserver = NumberServer::new(numthreads);
    eprintln!("Binding NumberServer");
    registry.bind("NumberServer", numserver);

    let t = Instant::now();
    run_threads(numthreads, port);

    let stub: NumberServerStub = reg
        .lookup("NumberServer")
        .expect("stub lookup failed")
        .into();
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
