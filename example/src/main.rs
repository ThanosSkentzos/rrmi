use clap::Parser;
use example::{
    client::{client, run_clients_local, run_clients_remote},
    server::server,
    utils::Utils,
    Config, NUM_CLIENTS_LOCAL,
};
use std::{process::exit, thread::sleep, time::Duration};

#[cfg(feature = "tracing")]
use tracing_chrome::ChromeLayerBuilder;
#[cfg(feature = "tracing")]
#[allow(unused)]
use tracing_subscriber::{prelude::*, registry::Registry};

fn main() {
    #[cfg(feature = "tracing")]
    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    #[cfg(feature = "tracing")]
    tracing_subscriber::registry().with(chrome_layer).init();

    let config = Config::parse();
    eprintln!("{config:?}");

    match config.local {
        true => {
            eprintln!("RUNNING LOCAL");
            run_local(config);
        }
        false => {
            eprintln!("RUNNING REMOTE");
            match config.liacs {
                false => run_remote_das(config),
                true => run_remote_liacs(config),
            }
        }
    }
}

pub fn run_local(config: Config) {
    server(run_clients_local, NUM_CLIENTS_LOCAL, config);
}

pub fn run_remote_liacs(config: Config) {
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
            config,
        );
    } else {
        let server_hostname = util.liacs_coordinator;
        sleep(Duration::from_secs(1));
        client(&server_hostname, config);
    }
}

pub fn run_remote_das(config: Config) {
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
            config,
        );
    } else {
        let server_hostname = util.slurm_coordinator;
        sleep(Duration::from_secs(1));
        client(&server_hostname, config);
    }
}
