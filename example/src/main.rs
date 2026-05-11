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
    #[arg(short, long, value_enum, default_value_t = Local::False)]
    local: Local,
    num_calls: usize,

    #[arg(long, value_enum, default_value_t = Liacs::False)]
    liacs: Liacs,
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
        Local::True => {
            eprintln!("RUNNING LOCAL");
            run_local(num_calls);
        }
        Local::False => {
            eprintln!("RUNNING REMOTE");
            match args.liacs {
                Liacs::False => run_remote_das(num_calls),
                Liacs::True => run_remote_liacs(num_calls),
            }
        }
    }
}
