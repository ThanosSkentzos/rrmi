use example::number_server::{run_local, run_remote};
use std::{env, process::exit};

fn main() {
    let local = true;
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
        run_local(num_calls);
    } else {
        run_remote(num_calls);
    }
}
