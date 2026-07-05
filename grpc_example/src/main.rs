use clap::{Parser, ValueEnum};
use grpc_example::{client::run_client, server::run_server, NUM_CLIENTS_LOCAL};
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
}

#[tokio::main]
async fn main() {
    let args = MyArgs::parse();
    match args.local {
        true => {
            eprintln!("RUNNING LOCAL");
            run_local().await;
        }
        false => {
            eprintln!("RUNNING REMOTE");
            run_remote();
        }
    }
}

async fn run_local() {
    let _sever_handle = tokio::spawn(async { run_server(NUM_CLIENTS_LOCAL).await });
    let mut set = JoinSet::new();
    for _ in 0..NUM_CLIENTS_LOCAL {
        set.spawn(async { run_client().await });
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

fn run_remote() {
    todo!()
}
