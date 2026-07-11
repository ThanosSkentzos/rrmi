use std::thread::sleep;
use std::{process::exit, time::Duration};

use clap::{Parser, ValueEnum};

#[cfg(feature = "infiniband")]
use grpc_example::utils::get_ib_hostname;
use grpc_example::{client::run_client, server::run_server, utils::Utils, NUM_CLIENTS_LOCAL};
use grpc_example::{NUM_HASH, NUM_NUMS, NUM_VECS};
use tokio::task::JoinSet;

#[derive(ValueEnum, Clone, Debug)]
enum Local {
    False,
    True,
}

#[derive(Parser, Debug)]
struct MyArgs {
    #[arg(long)]
    local: bool,

    #[arg(long, default_value_t = NUM_NUMS)]
    num_nums: usize,

    #[arg(long, default_value_t = NUM_VECS)]
    num_vecs: usize,

    #[arg(long, default_value_t = NUM_HASH)]
    num_hash: usize,
}

#[tokio::main]
async fn main() {
    let args = MyArgs::parse();
    let num_nums = args.num_nums;
    let num_vecs = args.num_vecs;
    let num_hash = args.num_hash;
    match args.local {
        true => {
            eprintln!("RUNNING LOCAL");
            run_local(num_nums, num_vecs, num_hash).await;
        }
        false => {
            eprintln!("RUNNING REMOTE");
            run_remote(num_nums, num_vecs, num_hash).await;
        }
    }
}

async fn run_local(num_nums: usize, num_vecs: usize, num_hash: usize) {
    let _sever_handle = tokio::spawn(async move { run_server(NUM_CLIENTS_LOCAL).await });
    let mut set = JoinSet::new();
    for _ in 0..NUM_CLIENTS_LOCAL {
        set.spawn(
            async move { run_client("http://localhost", num_nums, num_vecs, num_hash).await },
        );
    }
    // _ = run_server(NUM_CLIENTS_LOCAL).await;

    let mut finished = 0;
    while let Some(_res) = set.join_next().await {
        match _res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => eprintln!("Client error: {e}"),
            Err(e) => eprintln!("task panicked: {e}"),
        }
        finished += 1;
        eprintln!("Clients finished {finished}/{NUM_CLIENTS_LOCAL}")
    }
    _ = _sever_handle.await;
}

async fn run_remote(num_nums: usize, num_vecs: usize, num_hash: usize) {
    let util = Utils::new();
    eprintln!("{util:?}");

    if util.slurm_nodes.len() < 2 {
        eprintln!("This application needs to be executed on at least 2 machines.\nexiting...");
        exit(1);
    }

    let num_clients: u8 = util.slurm_nodes.len() as u8 - 1;
    let hostname = &util.slurm_coordinator.to_string();
    #[cfg(feature = "infiniband")]
    let hostname = get_ib_hostname(&hostname);
    let hostname = format!("http://{}", hostname);
    if util.am_i_slurm_coordinator() {
        _ = run_server(num_clients).await;
    } else {
        sleep(Duration::from_secs(1));
        _ = run_client(&hostname, num_nums, num_vecs, num_hash).await;
    }
}
