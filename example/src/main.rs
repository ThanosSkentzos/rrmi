use clap::{Parser, ValueEnum};
use example::{
    client::{client, run_clients_local, run_clients_remote},
    server::server,
    utils::Utils,
    NUM_CLIENTS_LOCAL, NUM_HASH, NUM_NUMS, NUM_VECS,
};
use std::{process::exit, thread::sleep, time::Duration};

#[cfg(feature = "tracing")]
use tracing_chrome::ChromeLayerBuilder;
#[cfg(feature = "tracing")]
#[allow(unused)]
use tracing_subscriber::{prelude::*, registry::Registry};

#[derive(ValueEnum, Clone, Debug)]
enum Local {
    False,
    True,
}

#[derive(ValueEnum, Clone, Debug)]
enum Liacs {
    False,
    True,
}

#[derive(Parser, Debug)]
struct MyArgs {
    #[arg(long)]
    local: bool,

    #[arg(long)]
    liacs: bool,

    #[arg(long, default_value_t = NUM_NUMS)]
    num_nums: usize,

    #[arg(long, default_value_t = NUM_VECS)]
    num_vecs: usize,

    #[arg(long, default_value_t = NUM_HASH)]
    num_hash: usize,
}
fn main() {
    #[cfg(feature = "tracing")]
    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    #[cfg(feature = "tracing")]
    tracing_subscriber::registry().with(chrome_layer).init();

    let args = MyArgs::parse();
    eprintln!("{args:?}");
    let num_nums = args.num_nums;
    let num_vecs = args.num_vecs;
    let num_hash = args.num_hash;

    match args.local {
        true => {
            eprintln!("RUNNING LOCAL");
            run_local(num_nums, num_vecs, num_hash);
        }
        false => {
            eprintln!("RUNNING REMOTE");
            match args.liacs {
                false => run_remote_das(num_nums, num_vecs, num_hash),
                true => run_remote_liacs(num_nums, num_vecs, num_hash),
            }
        }
    }
}

pub fn run_local(num_nums: usize, num_vecs: usize, num_hash: usize) {
    server(
        run_clients_local,
        NUM_CLIENTS_LOCAL,
        num_nums,
        num_vecs,
        num_hash,
    );
}

pub fn run_remote_liacs(num_nums: usize, num_vecs: usize, num_hash: usize) {
    let util = Utils::new();
    eprintln!("{util:?}");

    if util.liacs_nodes.len() < 2 {
        eprintln!("This application needs to be executed on at least 2 machines.\nexiting...");
        exit(1);
    }

    if util.am_i_liacs_coordinator() {
        server(
            run_clients_remote,
            (util.liacs_nodes.len() - 1) as u8,
            num_nums,
            num_vecs,
            num_hash,
        );
    } else {
        let server_hostname = util.liacs_coordinator;
        sleep(Duration::from_secs(1));
        client(&server_hostname, NUM_NUMS, NUM_VECS, NUM_HASH);
    }
}

pub fn run_remote_das(num_nums: usize, num_vecs: usize, num_hash: usize) {
    let util = Utils::new();
    eprintln!("{util:?}");

    if util.slurm_nodes.len() < 2 {
        eprintln!("This application needs to be executed on at least 2 machines.\nexiting...");
        exit(1);
    }

    if util.am_i_slurm_coordinator() {
        server(
            run_clients_remote,
            (util.slurm_nodes.len() - 1) as u8,
            num_nums,
            num_vecs,
            num_hash,
        );
    } else {
        let server_hostname = util.slurm_coordinator;
        sleep(Duration::from_secs(1));
        client(&server_hostname, num_nums, num_vecs, num_hash);
    }
}
