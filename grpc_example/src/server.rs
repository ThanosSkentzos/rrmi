use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use tokio::sync::{Barrier, Mutex, Notify};
use tonic::{transport::Server, Request, Response, Status};

use crate::experiment::benchmark_server::{Benchmark, BenchmarkServer};
use crate::experiment::{
    DoneHashRequest, DoneNumRequest, DoneVecRequest, HashRequest, HashResponse, NullResponse,
    NumRequest, NumResponse, VecRequest, VecResponse,
};
use crate::utils::get_my_hostname;
use crate::VEC_LEN;

#[derive(Debug)]
pub struct NumberServer {
    total_clients: u8,
    number: AtomicU32,
    bar: Barrier,
    time_num: Mutex<Duration>,
    time_vec: Mutex<Duration>,
    time_hash: Mutex<Duration>,
    num_clients_done: AtomicU32,
    pub shutdown: Arc<Notify>,
    hashmap_total_size: Mutex<usize>,
}
#[tonic::async_trait]
impl Benchmark for NumberServer {
    async fn inc_num(
        &self,
        _request: Request<NumRequest>,
    ) -> Result<Response<NumResponse>, Status> {
        // println!("Got a request: {:?}", _request);

        self.number.fetch_add(1, SeqCst);
        let number = self.number.load(SeqCst);
        let reply = NumResponse { number };

        Ok(Response::new(reply))
    }

    async fn send_vec(
        &self,
        _request: Request<VecRequest>,
    ) -> Result<Response<VecResponse>, Status> {
        // println!("Got a request with a vector of len {}", _request.into_inner().vector.iter().count());
        let reply = VecResponse {};
        Ok(Response::new(reply))
    }

    async fn send_hashmap(
        &self,
        _request: Request<HashRequest>,
    ) -> Result<Response<HashResponse>, Status> {
        // println!("Got a request with a hashmap of {} elements", _request.into_inner().hashmap.keys().count());
        let reply = HashResponse {};
        Ok(Response::new(reply))
    }

    async fn set_done_num(
        &self,
        request: Request<DoneNumRequest>,
    ) -> Result<Response<NullResponse>, Status> {
        let time = request.into_inner().time_num;
        // println!("Got a done_num request");
        let this_clients_time = Duration::from_secs_f32(time);
        let mut time_num = self.time_num.lock().await;
        *time_num += this_clients_time;
        self.num_clients_done.fetch_add(1, SeqCst);
        let num_done = self.num_clients_done.load(SeqCst);
        eprintln!("{num_done}");

        let reply = NullResponse {};
        Ok(Response::new(reply))
    }

    async fn set_done_vec(
        &self,
        request: Request<DoneVecRequest>,
    ) -> Result<Response<NullResponse>, Status> {
        let time = request.into_inner().time_vec;
        // println!("Got a done_vec request");

        let this_clients_time = Duration::from_secs_f32(time);
        let mut time_vec = self.time_vec.lock().await;
        *time_vec += this_clients_time;
        self.num_clients_done.fetch_add(1, SeqCst);
        let num_done = self.num_clients_done.load(SeqCst);
        eprintln!("{num_done}");

        let reply = NullResponse {};
        Ok(Response::new(reply))
    }

    async fn set_done_hash(
        &self,
        request: Request<DoneHashRequest>,
    ) -> Result<Response<NullResponse>, Status> {
        let request = request.into_inner();
        let time = request.time_hash;
        let hashmap_size = request.hashmap_size;
        let num_nums = request.num_nums as usize;
        let num_vecs = request.num_vecs as usize;
        let num_hash = request.num_hash as usize;
        // println!("Got a done_hash request");

        let this_clients_time = Duration::from_secs_f32(time);
        *self.time_hash.lock().await += this_clients_time;
        *self.hashmap_total_size.lock().await += hashmap_size as usize;

        self.num_clients_done.fetch_add(1, SeqCst);
        let num_done = self.num_clients_done.load(SeqCst);
        eprintln!("{num_done}/{}", 3 * self.total_clients as u32);
        if num_done == self.total_clients as u32 * 3 {
            self.print_results(num_nums, num_vecs, num_hash).await;
            sleep(Duration::from_secs(1));
            self.shutdown.notify_one();
            //TODO: print all statistics
        }

        let reply = NullResponse {};
        Ok(Response::new(reply))
    }

    async fn barrier(
        &self,
        _request: Request<NullResponse>,
    ) -> Result<Response<NullResponse>, Status> {
        self.bar.wait().await;
        let reply = NullResponse {};
        Ok(Response::new(reply))
    }
}

impl NumberServer {
    fn new(total_clients: u8) -> Self {
        Self {
            total_clients,
            number: AtomicU32::new(0),
            bar: Barrier::new(total_clients as usize),
            time_num: Mutex::new(Duration::new(0, 0)),
            time_vec: Mutex::new(Duration::new(0, 0)),
            time_hash: Mutex::new(Duration::new(0, 0)),
            num_clients_done: AtomicU32::new(0),
            shutdown: Arc::new(Notify::default()),
            hashmap_total_size: Mutex::new(0),
        }
    }

    async fn get_num_info(&self) -> Duration {
        let time = self.time_num.lock().await;
        time.clone()
    }

    async fn get_arr_info(&self) -> Duration {
        let time = self.time_vec.lock().await;
        time.clone()
    }

    async fn get_hashmap_info(&self) -> (Duration, usize) {
        let time = self.time_hash.lock().await;
        let size = self.hashmap_total_size.lock().await;
        (time.clone(), size.clone())
    }

    async fn print_results(&self, num_nums: usize, num_vecs: usize, num_hash: usize) {
        eprintln!("================= SERVER =================");

        let num_clients = self.total_clients;
        let final_num = self.number.load(SeqCst);

        let num_time = self.get_num_info().await;
        let num_size = size_of_val(&final_num);
        let num_count = num_clients as usize * num_nums;
        print_statistics(num_clients, "Sequence", num_time, num_count, num_size);

        let vecs_time = self.get_arr_info().await;
        let vec_size = size_of::<f64>() * VEC_LEN;
        let vec_count = num_clients as usize * num_vecs;
        print_statistics(num_clients, "Vector", vecs_time, vec_count, vec_size);

        let (hash_time, hashmaps_size) = self.get_hashmap_info().await;
        let hash_count = num_clients as usize * num_hash;
        let hash_avg_size = hashmaps_size / hash_count;
        print_statistics(num_clients, "Hashmap", hash_time, hash_count, hash_avg_size);
    }
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

pub async fn run_server(
    num_clients: u8,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    eprintln!("{} starting server", get_my_hostname());
    let addr = "[::]:50051".parse()?;
    let server = NumberServer::new(num_clients);
    let shutdown = server.shutdown.clone();
    Server::builder()
        .tcp_nodelay(true)
        .add_service(BenchmarkServer::new(server))
        .serve_with_shutdown(addr, async move {
            shutdown.notified().await;
            eprintln!("Done, shutting down")
        })
        .await?;

    Ok(())
}
