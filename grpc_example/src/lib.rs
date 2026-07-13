pub mod experiment {
    tonic::include_proto!("numberserver");
}
pub static NUM_NUMS: usize = 1_000;
pub static NUM_VECS: usize = 5;
pub static NUM_HASH: usize = 5;
pub static NUM_CLIENTS_LOCAL: u8 = 2;
pub static REG_PORT: u16 = 1099;

pub static HASHMAP_LEN: usize = 100_000;
pub static VEC_LEN: usize = 200_000;

pub mod client;
pub mod server;
pub mod utils;

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
enum Local {
    False,
    True,
}

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[arg(long)]
    pub local: bool,

    #[arg(long)]
    pub liacs: bool,

    #[arg(long, default_value_t = NUM_NUMS)]
    pub num_nums: usize,

    #[arg(long, default_value_t = NUM_VECS)]
    num_vecs: usize,

    #[arg(long, default_value_t = NUM_HASH)]
    num_hash: usize,

    #[arg(long, default_value_t = VEC_LEN)]
    vec_len: usize,

    #[arg(long, default_value_t = HASHMAP_LEN)]
    hash_len: usize,
}
