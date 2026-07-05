pub mod experiment {
    tonic::include_proto!("numberserver");
}
pub static NUM_NUMS: usize = 1_000;
pub static NUM_VECS: usize = 1;
pub static NUM_HASH: usize = 1;
pub static NUM_CLIENTS_LOCAL: u8 = 2;
pub static REG_PORT: u16 = 1099;

pub static HASHMAP_LEN: usize = 100_000;
pub static VEC_LEN: usize = 500_000;

pub mod client;
pub mod server;
