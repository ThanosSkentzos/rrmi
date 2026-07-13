use std::thread::sleep;
use std::{process::exit, time::Duration};

use clap::Parser;

#[cfg(feature = "infiniband")]
use grpc_example::utils::get_ib_hostname;
use grpc_example::Config;
use grpc_example::{client::run_client, server::run_server, utils::Utils, NUM_CLIENTS_LOCAL};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let args = Config::parse();
    match args.local {
        true => {
            eprintln!("RUNNING LOCAL");
            run_local(args).await;
        }
        false => {
            eprintln!("RUNNING REMOTE");
            run_remote(args).await;
        }
    }
}

async fn run_local(config: Config) {
    let _sever_handle = tokio::spawn(async move { run_server(NUM_CLIENTS_LOCAL).await });
    let mut set = JoinSet::new();
    for _ in 0..NUM_CLIENTS_LOCAL {
        let config_cloned = config.clone();
        set.spawn(async move { run_client("http://localhost", config_cloned).await });
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

async fn run_remote(config: Config) {
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
        _ = run_client(&hostname, config).await;
    }
}
