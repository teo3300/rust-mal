// io lib to read input and print output
use std::env::args;

mod core;
mod env;
mod eval;
mod mal_tests;
mod parse_tools;
mod printer;
mod reader;
mod step6_file;
mod types;

use core::ns_init;
use parse_tools::{interactive, load_conf, load_core, load_file, set_home_path};

fn main() {
    // Initialize ns environment
    let reply_env = ns_init();

    // Set the "MAL_HOME" symbol to the specified directory or the default one
    set_home_path(&reply_env);
    // load "$MAL_HOME/core.mal"
    load_core(&reply_env);

    // Load config files ($MAL_HOME/config.mal, or default $HOME/.config/mal/config.mal)
    load_conf(&reply_env);

    // load all files passed as arguments
    args().collect::<Vec<String>>()[1..].iter().for_each(|f| {
        if let Err(e) = load_file(f, &reply_env) {
            println!("{}", e.message())
        }
    });

    interactive(reply_env);
}
