// io lib to read input and print output
use std::io::{self, Write};

mod env;
mod eval;
mod printer;
mod reader;
mod types;
use types::env_init;

use env::Env;

mod step3_env;
use step3_env::rep;

fn main() {
    let mut num = 0;
    let mut reply_env = Env::new(None);
    env_init(&mut reply_env);

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
                match rep(&input, &mut reply_env) {
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
