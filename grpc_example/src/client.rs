use std::collections::HashMap;
use std::time::Instant;

use crate::experiment::benchmark_client::BenchmarkClient;
use crate::experiment::{
    DoneHashRequest, DoneNumRequest, DoneVecRequest, NullResponse, NumRequest, VecRequest,
};
use crate::utils::get_my_hostname;
use crate::Config;

use tonic::transport::Endpoint;
use tonic::Request;

use crate::experiment::HashRequest;

pub async fn run_client(
    hostname: &str,
    config: Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{hostname}:50051");
    eprintln!("{} starting client -> {url}", get_my_hostname());
    let endpoint = Endpoint::from_shared(url).unwrap().tcp_nodelay(true);
    let mut client = BenchmarkClient::connect(endpoint).await?;

    let (vector, hashmap, hashmap_size) = prep_data(&config);
    let num_start = Instant::now();

    for _ in 0..config.num_nums {
        let request = Request::new(NumRequest {});
        let _response = client.inc_num(request).await?;
    }
    let time_num = num_start.elapsed().as_secs_f32();
    let request = Request::new(DoneNumRequest { time_num });
    let _response = client.set_done_num(request).await?;
    eprintln!("got num response: {_response:?}");

    let request = Request::new(NullResponse {});
    let _ = client.barrier(request).await?;

    // print_statistics(1, "Sequence", time, NUM_NUMS, size_of_val(&final_num));

    // VECTOR
    let vec_start = Instant::now();
    for _ in 0..config.num_vecs {
        let request = Request::new(VecRequest {
            vector: vector.clone(),
        });
        let _response = client.send_vec(request).await?;
    }
    eprintln!("got vec response{_response:?}");
    let time_vec = vec_start.elapsed().as_secs_f32();
    let request = Request::new(DoneVecRequest { time_vec });
    let _ = client.set_done_vec(request).await?;
    let request = Request::new(NullResponse {});
    let _ = client.barrier(request).await?;

    // print_statistics(1, "Vector", vec_time, NUM_VECS, size_of::<f64>() * VEC_LEN);

    // HASHMAP
    let hash_start = Instant::now();
    for _ in 0..config.num_hash {
        let request = Request::new(HashRequest {
            hashmap: hashmap.clone(),
        });
        let _response = client.send_hashmap(request).await?;
    }
    eprintln!("got hash reponse {_response:?}");
    let time_hash = hash_start.elapsed().as_secs_f32();
    let request = Request::new(DoneHashRequest {
        time_hash,
        hashmap_size: hashmap_size as u32,
        num_nums: config.num_nums as u32,
        num_vecs: config.num_vecs as u32,
        num_hash: config.num_hash as u32,
        vec_len: config.vec_len as u32,
    });
    let _resp = client.set_done_hash(request).await?;

    // print_statistics(1, "HashMap", hash_time, NUM_HASH, hashmap_size);
    Ok(())
}

fn prep_data(config: &Config) -> (Vec<f64>, HashMap<String, String>, usize) {
    let vector: Vec<f64> = (0..config.vec_len).map(|_| rand::random::<f64>()).collect();
    let mut hashmap = HashMap::<String, String>::new();
    let mut hashmap_size: usize = 0;
    for i in 0..config.hash_len {
        let value = format!("{:.10}f", rand::random::<f64>());
        let key = format!("{i}");
        hashmap_size += key.len() + value.len();
        hashmap.insert(key, value);
    }
    (vector, hashmap, hashmap_size)
}
