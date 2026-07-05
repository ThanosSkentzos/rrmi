use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;

use tonic::{transport::Server, Request, Response, Status};

// use hello_world::greeter_server::{Greeter, GreeterServer};
// use hello_world::{HelloReply, HelloRequest};

// pub mod hello_world {
//     tonic::include_proto!("helloworld");
// }

pub mod experiment {
    tonic::include_proto!("numberserver");
}
use experiment::benchmark_server::{Benchmark, BenchmarkServer};
use experiment::{NumRequest, NumResponse, VecRequest, VecResponse};

use crate::experiment::{HashRequest, HashResponse};

#[derive(Debug, Default)]
pub struct NumberServer {
    number: AtomicU32,
}

#[tonic::async_trait]
impl Benchmark for NumberServer {
    async fn inc_num(&self, _request: Request<NumRequest>) -> Result<Response<NumResponse>, Status> {
        // println!("Got a request: {:?}", request);

        self.number.fetch_add(1, SeqCst);
        let number = self.number.load(SeqCst);
        let reply = NumResponse { number: number };

        Ok(Response::new(reply))
    }

    async fn send_vec(
        &self,
        _request: Request<VecRequest>,
    ) -> Result<Response<VecResponse>, Status> {
        // println!("Got a request: {:?}", request);
        let reply = VecResponse {};
        Ok(Response::new(reply))
    }

    async fn send_hashmap(
        &self,
        _request: Request<HashRequest>,
    ) -> Result<Response<HashResponse>, Status> {
        // println!("Got a request: {:?}", request);
        let reply = HashResponse {};
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = NumberServer::default();

    Server::builder()
        .tcp_nodelay(true)
        .add_service(BenchmarkServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
