// io lib to read input and print output
use std::env::args;

mod core;
mod env;
mod eval;
mod parse_tools;
mod printer;
mod reader;
mod step4_if_fn_do;
mod types;

use core::ns_init;
use parse_tools::{interactive, load_file};

fn main() {
    let reply_env = ns_init();

    // setup env
    //let args: Vec<String> = args().collect();
    args().collect::<Vec<String>>()[1..]
        .iter()
        .for_each(|f| load_file(f, &reply_env));

    interactive(reply_env);
}
