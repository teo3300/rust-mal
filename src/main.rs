// io lib to read input and print output
use std::env::args;

mod core;
mod env;
mod eval;
mod parse_tools;
mod printer;
mod reader;
mod step4_if_fn_do;
mod tests;
mod types;

use core::ns_init;
use parse_tools::{interactive, load_file};

fn main() {
    // Initialize ns environment
    let reply_env = ns_init();

    // load all files passed as arguments
    args().collect::<Vec<String>>()[1..].iter().for_each(|f| {
        if let Err(e) = load_file(f, &reply_env) {
            println!("{}", e.message())
        }
    });

    interactive(reply_env);
}
