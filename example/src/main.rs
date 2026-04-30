use example::number_server::{run_local, run_remote};
use std::{env, process::exit};

fn main() {
    let local = false;
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Too few arguments, usage: ./example <number_calls>");
        exit(1);
    }
    let arg = &args[1];
    let num_calls = arg
        .parse::<usize>()
        .expect(&format!("Error parsing argument {arg} as usize"));
    if local {
        eprintln!("RUNNING LOCAL");
        run_local(num_calls);
    } else {
        eprintln!("RUNNING REMOTE");
        run_remote(num_calls);
    }
}
