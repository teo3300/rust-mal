// io lib to read input and print output
use std::io::{self, Write};

mod printer;
mod reader;
mod types;

mod step1_read_print;
use step1_read_print::rep;

fn main() -> io::Result<()> {
    let mut num = 0;
    loop {
        let mut input = String::new();
        loop {
            print!("user> ");
            // Flush the prompt to appear before command
            let _ = io::stdout().flush();

            // Read line to compose program inpug
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;

            input.push_str(&line);
            if input == "\n" {
                break;
            }

            // Perform rep on whole available input
            match rep(&input) {
                Ok(output) => {
                    num += 1;
                    println!("[{}]> {}", num, output);
                }
                Err(err) => {
                    if line == "\n" {
                        num += 1;
                        println!("; [{}]> {}", num, err);
                    } else {
                        continue;
                    }
                }
            };
            break;
        }
    }
}
