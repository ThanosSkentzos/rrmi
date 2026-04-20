use example::number_server::{run_local, run_remote};
use std::env;

fn main() {
    let local = false;
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        eprintln!("Usage: ./example <number_calls>")
    }
    let arg = &args[1];
    let num_calls = arg
        .parse::<usize>()
        .expect(&format!("Error parsing argument {arg} as usize"));
    if local {
        run_local(num_calls);
    } else {
        run_remote(num_calls);
    }
}
