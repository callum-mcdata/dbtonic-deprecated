// dbtonic-cli/src/main.rs
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbtonic::run(args);
}