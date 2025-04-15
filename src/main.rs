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
use parse_tools::{interactive, load_file, load_home_file, pre_load, print_banner, set_home_path};

fn main() {
    // Initialize ns environment
    let reply_env = ns_init();

    // Set the "MAL_HOME" symbol to the specified directory or the default one
    set_home_path(&reply_env);
    // load "$MAL_HOME/core.mal" [warn: true] since this has some core functionalities
    load_home_file("core.mal", &reply_env, true);
    // Load config files ($MAL_HOME/config.mal, or default $HOME/.config/mal/config.mal)
    // [warn: false] since this file is optional
    // replaced by (ok? '(reload-config)) at the end of core.mal
    //  - I used this to overwrite BANNER to prevent it from displaying
    //    based on conf
    //load_home_file("config.mal", &reply_env, false);

    // load ~~all files~~ first file passed as arguments
    //args().collect::<Vec<String>>()[1..].iter().for_each(|f| {
    //    if let Err(e) = load_file(f, &reply_env) {
    //        eprintln!("{}", e.message())
    //    }
    //});
    {
        let argv = args().collect::<Vec<String>>();
        pre_load(&argv, &reply_env);
        if argv.len() > 1 {
            if let Err(e) = load_file(&argv[1], &reply_env) {
                eprintln!("{}", e.message())
            }
        }
    }

    print_banner(&reply_env);

    interactive(reply_env);
}
