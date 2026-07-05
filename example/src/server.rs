use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Barrier, Condvar};
use std::time::Instant;
use std::{
    sync::atomic::{AtomicBool, AtomicU32, AtomicU8},
    sync::Mutex,
    thread,
    time::Duration,
};

use crate::{REG_PORT, VEC_LEN};
use rrmi::{create_registry, get_registry, remote::RemoteObject};
use rrmi_macros::remote_object;
use thousands::Separable;
//=============================TRACING============================

#[cfg(feature = "tracing")]
use tracing::{instrument, span, Level};
#[cfg(feature = "tracing")]
use tracing_chrome::ChromeLayerBuilder;
#[cfg(feature = "tracing")]
#[allow(unused)]
use tracing_subscriber::{prelude::*, registry::Registry};

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
    num_clients_done: AtomicUsize,
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
    #[remote]
    fn get_clients_done(&self) -> usize {
        self.num_clients_done.load(SeqCst)
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

pub fn server(
    experiment: fn(u8, usize, usize, usize),
    num_clients: u8,
    num_nums: usize,
    num_vecs: usize,
    num_hash: usize,
) {
    // CREATE REGISTRY
    let port = REG_PORT;
    eprintln!("Creating Registry");
    let registry = create_registry(port);

    // CREATE OBJECT
    let numserver = NumberServer::new(num_clients);
    eprintln!("Binding NumberServer");
    let (num_server, _id) = registry.bind("NumberServer", numserver);

    // RUN EXPERIMENT
    let t = Instant::now();
    eprintln!("Running experiment with {num_clients} clients and {num_nums} numbers sent");
    experiment(num_clients, num_nums, num_vecs, num_hash);
    let time = t.elapsed();

    // FINAL NUMBER
    eprintln!("Getting RegistryStub");
    let reg = get_registry("localhost", port);
    let stub: NumberServerStub = reg
        .lookup("NumberServer")
        .expect("stub lookup failed")
        .into();
    let final_num = stub.inc_num().expect("stub get_num failed");
    let mutex = stub
        .get_barrier_count()
        .expect("stub get_barrier_count failed");

    eprintln!(
        "Total count atomic: {} & mutex: {}, clients_done: {}",
        final_num.separate_with_underscores(),
        mutex.separate_with_underscores(),
        num_server.get_clients_done()
    );
    // STATISTICS
    eprintln!("================= SERVER =================");
    eprintln!("Total time|count: {time:?}|{final_num}");
    eprintln!("Average: {:?}", time / final_num);

    let num_time = num_server.get_num_info();
    let num_size = size_of_val(&final_num);
    let num_count = num_clients as usize * num_nums;
    print_statistics(num_clients, "Sequence", num_time, num_count, num_size);

    let vecs_time = num_server.get_arr_info();
    let vec_size = size_of::<f64>() * VEC_LEN;
    let vec_count = num_clients as usize * num_vecs;
    print_statistics(num_clients, "Vector", vecs_time, vec_count, vec_size);

    let (hash_time, hashmaps_size) = num_server.get_hashmap_info();
    let hash_count = num_clients as usize * num_hash;
    let hash_avg_size = hashmaps_size / hash_count;
    print_statistics(num_clients, "Hashmap", hash_time, hash_count, hash_avg_size);
}

fn print_statistics(
    num_clients: u8,
    label: &str,
    total_time: Duration,
    total_count: usize,
    avegare_size: usize,
) {
    let bytes_to_bits: f32 = 8.0;
    let average_rtt = total_time / total_count as u32;
    let throughput = bytes_to_bits * avegare_size as f32 / average_rtt.as_secs_f32();
    eprintln!("================= {label} =================");
    eprintln!("Total time|calls server: {total_time:?}|{total_count}");
    eprintln!("Average roundtrip: {average_rtt:?}");
    eprintln!("Average lat: {:?}", average_rtt / 2);
    eprintln!("Average throughput: {:?} bps", throughput);

    println!("NClients,Type,TotalCalls,Time,MicrosPerCall,Latency,Throughput");
    let micros_in_sec = 1_000_000.0;
    println!(
        "{},{},{},{},{},{},{}",
        num_clients,
        label,
        total_count,
        total_time.as_secs_f32() * micros_in_sec,
        average_rtt.as_secs_f32() * micros_in_sec,
        average_rtt.as_secs_f32() * micros_in_sec / 2.0,
        throughput
    )
}
