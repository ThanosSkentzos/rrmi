use std::collections::HashMap;
use std::thread::sleep;
use std::time::Instant;
use std::vec;
use std::{thread, time::Duration};

use rrmi::get_registry;

//=============================TRACING============================
#[cfg(feature = "tracing")]
use tracing::{instrument, span, Level};
#[cfg(feature = "tracing")]
use tracing_chrome::ChromeLayerBuilder;
#[cfg(feature = "tracing")]
#[allow(unused)]
use tracing_subscriber::{prelude::*, registry::Registry};

use crate::server::NumberServerStub;
use crate::{HASHMAP_LEN, REG_PORT, VEC_LEN};

#[allow(unused)]
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
pub fn client(host: &str, nums: usize, vecs: usize, hashmaps: usize) {
    let reg = get_registry(host, REG_PORT);
    let stub: NumberServerStub = reg
        .lookup("NumberServer")
        .expect("stub lookup failed")
        .into();
    let (vector, hashmap, hashmap_size) = prep_data();
    let _ = stub.barrier_mutex();
    send_nums(&stub, nums);

    let _ = stub.barrier_mutex();
    send_vecs(&stub, vecs, &vector);

    let _ = stub.barrier_mutex();
    send_hashmaps(&stub, hashmaps, &hashmap, hashmap_size);
}

#[cfg_attr(feature = "tracing", instrument)]
pub fn run_clients_local(num_clients: u8, num_nums: usize, num_vecs: usize, num_hash: usize) {
    let mut handles = vec![];
    for i in 0..num_clients {
        let handle = thread::Builder::new()
            .name(format!("Stub{i}"))
            .spawn(move || {
                client("localhost", num_nums, num_vecs, num_hash);
            })
            .expect("Could not spawn thread.");
        handles.push(handle);
    }
    for handle in handles {
        handle.join().expect("Could not join handle");
    }
}

#[cfg_attr(feature = "tracing", instrument)]
pub fn run_clients_remote(num_clients: u8, num_nums: usize, num_vecs: usize, num_hash: usize) {
    eprintln!("waiting for {num_clients} clients to finish {num_nums} of inc_num, {num_vecs} of send_large_vec and {num_hash} send_hashmap");
    let reg = get_registry("localhost", REG_PORT);
    let stub: NumberServerStub = reg
        .lookup("NumberServer")
        .expect("stub lookup failed")
        .into();
    let mut done = false;
    let mut prev: usize;
    let mut num_done: usize = 0;
    while !done {
        prev = num_done;
        num_done = stub.get_clients_done().unwrap();
        if prev != num_done {
            eprintln!("Done clients increased to {num_done}")
        }
        if num_done == 3 * num_clients as usize {
            done = true;
        }
        sleep(Duration::from_millis(10));
    }
}
