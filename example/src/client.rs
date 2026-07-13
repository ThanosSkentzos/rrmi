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
use crate::{Config, REG_PORT};

#[allow(unused)]
#[cfg_attr(feature = "tracing", instrument)]
fn prep_data(vec_len: usize, hash_len: usize) -> (Vec<f64>, HashMap<String, String>, usize) {
    #[cfg(feature = "tracing")]
    let span = span!(Level::TRACE, "vec");
    #[cfg(feature = "tracing")]
    let _enter = span.enter();
    let vector: Vec<f64> = (0..vec_len).map(|_| rand::random::<f64>()).collect();
    #[cfg(feature = "tracing")]
    drop(_enter);
    #[cfg(feature = "tracing")]
    let span = span!(Level::TRACE, "hashmap");
    #[cfg(feature = "tracing")]
    let _enter = span.enter();
    let mut hashmap = HashMap::<String, String>::new();
    let mut hashmap_size: usize = 0;
    for i in 0..hash_len {
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
    // Warmup
    let _ = stub.barrier_mutex();
    for _ in 0..100 {
        _ = stub.inc_num().unwrap();
    }
    let _ = stub.barrier_mutex();
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
pub fn client(host: &str, config: Config) {
    let reg = get_registry(host, REG_PORT);
    let stub: NumberServerStub = reg
        .lookup("NumberServer")
        .expect("stub lookup failed")
        .into();
    let (vector, hashmap, hashmap_size) = prep_data(config.vec_len, config.hash_len);
    send_nums(&stub, config.num_nums);

    let _ = stub.barrier_mutex();
    send_vecs(&stub, config.num_vecs, &vector);

    let _ = stub.barrier_mutex();
    send_hashmaps(&stub, config.num_hash, &hashmap, hashmap_size);
}

#[cfg_attr(feature = "tracing", instrument)]
pub fn run_clients_local(num_clients: u8, config: Config) {
    let mut handles = vec![];
    for i in 0..num_clients {
        let config_cloned = config.clone();
        let handle = thread::Builder::new()
            .name(format!("Stub{i}"))
            .spawn(move || {
                client("localhost", config_cloned);
            })
            .expect("Could not spawn thread.");
        handles.push(handle);
    }
    for handle in handles {
        handle.join().expect("Could not join handle");
    }
}

#[cfg_attr(feature = "tracing", instrument)]
pub fn run_clients_remote(num_clients: u8, config: Config) {
    eprintln!("waiting for {num_clients} clients to finish {} of inc_num, {} of send_large_vec and {} send_hashmap",config.num_nums,config.num_vecs,config.num_hash);
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
