use example::number_server::{run_local, run_remote_das, run_remote_liacs};

#[cfg(feature = "tracing")]
use tracing_chrome::ChromeLayerBuilder;
#[cfg(feature = "tracing")]
#[allow(unused)]
use tracing_subscriber::{prelude::*, registry::Registry};

use clap::{Parser, ValueEnum};

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

    #[arg(default_value_t=10_000)]
    num_calls: usize,

    #[arg(long)]
    liacs: bool,
}
fn main() {
    #[cfg(feature = "tracing")]
    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    #[cfg(feature = "tracing")]
    tracing_subscriber::registry().with(chrome_layer).init();

    let args = MyArgs::parse();
    eprintln!("{args:?}");
    let num_calls = args.num_calls;
    match args.local {
        true => {
            eprintln!("RUNNING LOCAL");
            run_local(num_calls);
        }
        false => {
            eprintln!("RUNNING REMOTE");
            match args.liacs {
                false => run_remote_das(num_calls),
                true => run_remote_liacs(num_calls),
            }
        }
    }
}
