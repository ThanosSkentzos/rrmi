use std::collections::HashMap;
use std::time::Instant;

use crate::experiment::benchmark_client::BenchmarkClient;
use crate::experiment::{
    DoneHashRequest, DoneNumRequest, DoneVecRequest, NullResponse, NumRequest, VecRequest,
};
use crate::{HASHMAP_LEN, NUM_HASH, NUM_NUMS, NUM_VECS, VEC_LEN};

use tonic::transport::Endpoint;
use tonic::Request;

use crate::experiment::HashRequest;

pub async fn run_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let endpoint = Endpoint::from_static("http://[::1]:50051").tcp_nodelay(true);
    let mut client = BenchmarkClient::connect(endpoint).await?;

    let (vector, hashmap, hashmap_size) = prep_data();
    let num_start = Instant::now();

    for _ in 0..NUM_NUMS {
        let request = Request::new(NumRequest {});
        let _response = client.inc_num(request).await?;
    }
    let time_num = num_start.elapsed().as_secs_f32();
    let request = Request::new(DoneNumRequest { time_num });
    let _response = client.set_done_num(request).await?;
    eprintln!("{_response:?}");

    let request = Request::new(NullResponse {});
    let _ = client.barrier(request).await?;

    // print_statistics(1, "Sequence", time, NUM_NUMS, size_of_val(&final_num));

    // VECTOR
    let request = Request::new(VecRequest { vector });
    let vec_start = Instant::now();
    let _response = client.send_vec(request).await?;
    eprintln!("{_response:?}");
    let time_vec = vec_start.elapsed().as_secs_f32();
    let request = Request::new(DoneVecRequest { time_vec });
    let _ = client.set_done_vec(request).await?;
    let request = Request::new(NullResponse {});
    let _ = client.barrier(request).await?;

    // print_statistics(1, "Vector", vec_time, NUM_VECS, size_of::<f64>() * VEC_LEN);

    // HASHMAP
    let request = Request::new(HashRequest { hashmap });
    let hash_start = Instant::now();
    let _response = client.send_hashmap(request).await?;
    eprintln!("{_response:?}");
    let time_hash = hash_start.elapsed().as_secs_f32();
    let request = Request::new(DoneHashRequest {
        time_hash,
        hashmap_size: hashmap_size as u32,
    });
    let _resp = client.set_done_hash(request).await?;

    // print_statistics(1, "HashMap", hash_time, NUM_HASH, hashmap_size);
    Ok(())
}

fn prep_data() -> (Vec<f64>, HashMap<String, String>, usize) {
    let vector: Vec<f64> = (0..VEC_LEN).map(|_| rand::random::<f64>()).collect();
    let mut hashmap = HashMap::<String, String>::new();
    let mut hashmap_size: usize = 0;
    for i in 0..HASHMAP_LEN {
        let value = format!("{:.10}f", rand::random::<f64>());
        let key = format!("{i}");
        hashmap_size += key.len() + value.len();
        hashmap.insert(key, value);
    }
    (vector, hashmap, hashmap_size)
}
