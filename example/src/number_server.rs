use std::collections::HashMap;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Barrier, Condvar};
use std::time::Instant;
use std::vec;
use std::{
    sync::atomic::{AtomicBool, AtomicU32, AtomicU8},
    sync::Mutex,
    thread,
    time::Duration,
};

use rrmi::{create_registry, get_registry, remote::RemoteObject};
use rrmi_macros::remote_object;
use thousands::Separable;
static HASHMAP_LEN: usize = 100_000;
static VEC_LEN: usize = 1_000_000;
//=============================TRACING============================

#[cfg(feature = "tracing")]
use tracing::{instrument, span, Level};
#[cfg(feature = "tracing")]
use tracing_chrome::ChromeLayerBuilder;
#[cfg(feature = "tracing")]
#[allow(unused)]
use tracing_subscriber::{prelude::*, registry::Registry};

use crate::utils::{self, Utils};

#[allow(unused)]
#[cfg_attr(feature = "tracing", derive(Debug))]
struct NumberServer {
    num_atomic: AtomicU32,
    num_mutex: Mutex<u32>,
    count: AtomicU8,
    total_clients: u8,
    barrier_on: AtomicBool,
    bar: Barrier,
    count_mut: Mutex<u8>,
    barrier_num: Mutex<usize>,
    condvar: Condvar,
    time_num: Mutex<Duration>,
    time_arr: Mutex<Duration>,
    time_hash: Mutex<Duration>,
    hashmap_total_size: Mutex<usize>,
    num_clients_done: AtomicU8,
}

#[remote_object]
impl NumberServer {
    #[cfg_attr(feature = "tracing", instrument)]
    fn new(total_clients: u8) -> Self {
        let num_atomic = 0.into();
        let num_mutex = Mutex::new(0);
        let count = 0.into();
        let barrier_on = AtomicBool::new(false);
        let bar = Barrier::new(total_clients as usize);
        let count_mut = Mutex::new(0);
        let barrier_num = Mutex::new(0);
        let condvar = Condvar::new();
        let time_num = Mutex::new(Duration::new(0, 0));
        let time_arr = Mutex::new(Duration::new(0, 0));
        let time_hash = Mutex::new(Duration::default());
        let hashmap_total_size = Mutex::new(0);
        let num_clients_done = 0.into();
        Self {
            num_atomic,
            num_mutex,
            count,
            total_clients,
            barrier_on,
            bar,
            count_mut,
            barrier_num,
            condvar,
            time_num,
            time_arr,
            time_hash,
            hashmap_total_size,
            num_clients_done,
        }
    }
    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn inc_num(&self) -> u32 {
        self.num_atomic.fetch_add(1, SeqCst);
        self.num_atomic.load(SeqCst)
    }

    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn send_large_vec(&self, data: Vec<f64>) -> () {
        eprintln!(
            "Received large vector of size: {}x{}B",
            data.len(),
            size_of::<f64>()
        )
    }

    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn send_hashmap(&self, data: HashMap<String, String>) -> () {
        eprintln!("Received hashmap with {} elements", data.len())
    }

    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn get_barrier_count(&self) -> u32 {
        let mut num2 = self.num_mutex.lock().unwrap();
        *num2 += 1;
        *num2
    }

    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn barrier_atomic(&self) -> () {
        self.barrier_on.store(true, SeqCst);
        self.count.fetch_add(1, SeqCst);
        let mut inside = self.count.load(SeqCst);
        if inside == self.total_clients {
            self.barrier_on.store(false, SeqCst);
            self.count.store(0, SeqCst);
        }
        while inside < self.total_clients {
            inside = self.count.load(SeqCst);
            let status = self.barrier_on.load(SeqCst);
            if status == false {
                break;
            }
            thread::sleep(Duration::from_micros(10));
        }
    }

    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn barrier_bar(&self) -> () {
        self.bar.wait();
    }

    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn barrier_mutex(&self) -> () {
        let barrier_num = self.barrier_num.lock().unwrap();
        let current_num = *barrier_num;
        let mut count = self.count_mut.lock().unwrap();
        *count += 1;
        let current_count = *count;
        drop(count);
        drop(barrier_num);
        if current_count < self.total_clients {
            let _res = self
                .condvar
                .wait_while(self.barrier_num.lock().unwrap(), |num| current_num == *num);
        } else {
            *self.count_mut.lock().unwrap() = 0;
            *self.barrier_num.lock().unwrap() += 1;
            self.condvar.notify_all();
        }
    }

    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn set_done_num(&self, time: Duration) -> () {
        let mut time_num = self.time_num.lock().expect("Could not get lock");
        *time_num += time;
        self.num_clients_done.fetch_add(1, SeqCst);
    }

    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn set_done_arr(&self, time: Duration) -> () {
        let mut time_arr = self.time_arr.lock().expect("Could not get lock");
        *time_arr += time;
        self.num_clients_done.fetch_add(1, SeqCst);
    }

    #[remote]
    #[cfg_attr(feature = "tracing", instrument)]
    fn set_done_hash(&self, time: Duration, size: usize) -> () {
        let mut time_hash = self.time_hash.lock().expect("Could not get lock");
        *time_hash += time;
        self.num_clients_done.fetch_add(1, SeqCst);
        let mut hashmap_total_size = self
            .hashmap_total_size
            .lock()
            .expect("Could not acquire lock");
        *hashmap_total_size += size
    }

    fn get_num_info(&self) -> Duration {
        let time = self.time_num.lock().expect("unable to acquire lock");
        time.clone()
    }

    fn get_arr_info(&self) -> Duration {
        let time = self.time_arr.lock().expect("unable to acquire lock");
        time.clone()
    }

    fn get_hashmap_info(&self) -> (Duration, usize) {
        let time = self.time_hash.lock().expect("unable to acquire lock");
        let size = self
            .hashmap_total_size
            .lock()
            .expect("Could not acquire lock");
        (time.clone(), size.clone())
    }
}

#[cfg_attr(feature = "tracing", instrument)]
fn prep_data() -> (Vec<f64>, HashMap<String, String>, usize) {
    #[cfg(feature = "tracing")]
    let span = span!(Level::TRACE, "vec");
    #[cfg(feature = "tracing")]
    let _enter = span.enter();
    let vector: Vec<f64> = (0..VEC_LEN).map(|_| rand::random::<f64>()).collect();
    #[cfg(feature = "tracing")]
    drop(_enter);
    #[cfg(feature = "tracing")]
    let span = span!(Level::TRACE, "hashmap");
    #[cfg(feature = "tracing")]
    let _enter = span.enter();
    let mut hashmap = HashMap::<String, String>::new();
    let mut hashmap_size: usize = 0;
    for i in 0..HASHMAP_LEN {
        let value = format!("{:.10}f", rand::random::<f64>());
        let key = format!("{i}");
        hashmap_size += key.len() + value.len();
        hashmap.insert(key, value);
    }
    #[cfg(feature = "tracing")]
    drop(_enter);
    (vector, hashmap, hashmap_size)
}
#[cfg_attr(feature = "tracing", instrument)]
fn send_nums(stub: &NumberServerStub, times: usize) {
    let start = Instant::now();
    for _ in 0..times {
        _ = stub.inc_num().unwrap();
    }
    let time = start.elapsed();
    stub.set_done_num(time).unwrap();
}

#[cfg_attr(feature = "tracing", instrument)]
fn send_vecs(stub: &NumberServerStub, times: usize, vector: &Vec<f64>) {
    let start = Instant::now();
    for _ in 0..times {
        _ = stub.send_large_vec(vector.clone()).unwrap();
    }
    let time = start.elapsed();
    _ = stub.set_done_arr(time);
}

#[cfg_attr(feature = "tracing", instrument)]
fn send_hashmaps(
    stub: &NumberServerStub,
    times: usize,
    hashmap: &HashMap<String, String>,
    hashmap_size: usize,
) {
    let start = Instant::now();
    for _ in 0..times {
        _ = stub.send_hashmap(hashmap.clone());
    }
    let time = start.elapsed();
    _ = stub.set_done_hash(time, hashmap_size);
}
#[cfg_attr(feature = "tracing", instrument)]
fn client(stub: &NumberServerStub, nums: usize, vecs: usize, hashmaps: usize) {
    let (vector, hashmap, hashmap_size) = prep_data();
    let _ = stub.barrier_mutex();
    send_nums(stub, nums);

    let _ = stub.barrier_mutex();
    send_vecs(stub, vecs, &vector);

    let _ = stub.barrier_mutex();
    send_hashmaps(stub, hashmaps, &hashmap, hashmap_size);
}

#[cfg_attr(feature = "tracing", instrument)]
fn run_clients(n: u8, port: u16, nums: usize, vecs: usize, hashmaps: usize) {
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
                client(&stub, nums, vecs, hashmaps);
            })
            .expect("Could not spawn thread.");
        handles.push(handle);
    }
    for handle in handles {
        handle.join().expect("Could not join handle");
    }
}

pub fn local_test() {
    #[cfg(feature = "tracing")]
    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    #[cfg(feature = "tracing")]
    tracing_subscriber::registry().with(chrome_layer).init();

    let port = 1099;
    let (num_clients, num_nums, num_vecs, num_hash) = (4, 1000, 1, 1);
    eprintln!("Creating Registry");
    let registry = create_registry(port);
    eprintln!("Getting RegistryStub");
    let reg = get_registry("localhost", port);

    let numserver = NumberServer::new(num_clients);
    eprintln!("Binding NumberServer");
    let (num_server, _id) = registry.bind("NumberServer", numserver);

    let t = Instant::now();
    run_clients(num_clients, port, num_nums, num_vecs, num_hash);
    let time = t.elapsed();

    let stub: NumberServerStub = reg
        .lookup("NumberServer")
        .expect("stub lookup failed")
        .into();
    let final_num = stub.inc_num().expect("stub get_num failed");
    let mutex = stub
        .get_barrier_count()
        .expect("stub get_barrier_count failed");

    eprintln!(
        "Total count atomic: {} & mutex: {}",
        final_num.separate_with_underscores(),
        mutex.separate_with_underscores()
    );

    eprintln!("================= SERVER =================");
    eprintln!("Total time|count: {time:?}|{final_num}");
    eprintln!("Average: {:?}", time / final_num);

    eprintln!("================= NUMBER =================");
    let nums_time = num_server.get_num_info();
    let num_size = size_of_val(&final_num);
    let num_count = num_clients as usize * num_nums;
    print_statistics(nums_time, num_count, num_size);

    eprintln!("================= VECTOR =================");
    let vecs_time = num_server.get_arr_info();
    let vec_size = size_of::<f64>() * VEC_LEN;
    let vec_count = num_clients as usize * num_vecs;
    print_statistics(vecs_time, vec_count, vec_size);

    eprintln!("================= HASHMAP =================");
    let (time_hash, hashmaps_size) = num_server.get_hashmap_info();
    let hashmap_count = num_clients as usize * num_hash;
    let hashmaps_avg_size = hashmaps_size / hashmap_count;
    print_statistics(time_hash, hashmap_count, hashmaps_avg_size);
}

fn print_statistics(total_time: Duration, total_count: usize, avegare_size: usize) {
    let bytes_to_bits: f32 = 8.0;
    let average_rtt = total_time / total_count as u32;
    let throughput = bytes_to_bits * avegare_size as f32 / average_rtt.as_secs_f32();
    eprintln!("Total time|calls server: {total_time:?}|{total_count}");
    eprintln!("Average roundtrip: {average_rtt:?}");
    eprintln!("Average lat: {:?}", average_rtt / 2);
    eprintln!("Average throughput: {:?} bps", throughput);
}

pub fn remote_test() {
    let util = Utils::new();
    eprintln!("{util:?}")
}
