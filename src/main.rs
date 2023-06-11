// io lib to read input and print output
use std::io::{self, Write};

mod envs;
mod mal_core;
mod printer;
mod reader;
mod types;

use envs::Env;

mod step2_eval;
use step2_eval::rep;

fn main() {
    let mut num = 0;
    let env = Env::new(None);

    loop {
        let mut input = String::new();
        loop {
            print!("user> ");
            // Flush the prompt to appear before command
            let _ = io::stdout().flush();

            // Read line to compose program inpug
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();

            input.push_str(&line);

            if input != "\n" {
                // Perform rep on whole available input
                match rep(&input, &env) {
                    Ok(output) => println!("[{}]> {}", num, output),
                    Err(err) => {
                        if line != "\n" {
                            continue;
                        }
                        println!("; [{}]> Error {}", num, err);
                    }
                }
                num += 1;
            }
            break;
        }
    }
}
