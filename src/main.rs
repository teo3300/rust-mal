// io lib to read input and print output
use std::env::args;

mod env;
mod eval;
mod parse_tools;
mod printer;
mod reader;
mod step4_if_fn_do;
mod types;
mod core;

use env::env_init;
use parse_tools::{load_file, interactive};

fn main() {
    let reply_env = env_init();

    // setup env
    let args: Vec<String> = args().collect();
    for filename in &args[1..] {
        let _ = load_file(filename, &reply_env);
    }

    interactive(reply_env);
}
