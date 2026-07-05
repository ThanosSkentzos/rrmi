pub mod experiment {
    tonic::include_proto!("numberserver");
}
use std::collections::HashMap;
use std::time::{Duration, Instant};

use experiment::benchmark_client::BenchmarkClient;
use experiment::{NumRequest, VecRequest};

use tonic::Request;
use tonic::transport::Endpoint;

use crate::experiment::HashRequest;

static HASHMAP_LEN: usize = 100_000;
static VEC_LEN: usize = 500_000;
static NUM_NUMS: usize = 10;
static NUM_VECS: usize = 1;
static NUM_HASH: usize = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = Endpoint::from_static("http://[::1]:50051").tcp_nodelay(true);
    let mut client = BenchmarkClient::connect(endpoint).await?;

    let (vector, hashmap, hashmap_size) = prep_data();
    let num_start = Instant::now();

    for _ in 0..NUM_NUMS - 1 {
        let request = Request::new(NumRequest {});
        let _response = client.inc_num(request).await?;
    }
    let time = num_start.elapsed();
    eprintln!("Totam Time: {time:?}");
    eprintln!("Num Time: {time:?}");

    let request = Request::new(NumRequest {});
    let response = client.inc_num(request).await?;
    let final_num = response.into_inner().number;
    println!("Final number = {final_num}");
    print_statistics(1, "Sequence", time, NUM_NUMS, size_of_val(&final_num));

    // VECTOR
    let request = Request::new(VecRequest { vector });
    let vec_start = Instant::now();
    let response = client.send_vec(request).await?;
    let vec_time = vec_start.elapsed();
    eprintln!("Vec Time: {vec_time:?}");

    println!("RESPONSE={:?}", response);
    print_statistics(1, "Vector", vec_time, NUM_VECS, size_of::<f64>()*VEC_LEN);

    // HASHMAP
    let request = Request::new(HashRequest { hashmap });
    let hash_start = Instant::now();
    let response = client.send_hashmap(request).await?;
    let hash_time = hash_start.elapsed();
    eprintln!("Vec Time: {hash_time:?}");

    println!("RESPONSE={:?}", response);
    print_statistics(1, "HashMap", hash_time, NUM_HASH, hashmap_size);
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
