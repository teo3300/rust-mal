// io lib to read input and print output
use std::io::{self, Write};

mod printer;
mod reader;
mod types;

mod step1_read_print;
use step1_read_print::rep;

fn main() -> io::Result<()> {
    loop {
        let mut input = String::new();
        loop {
            print!("user> ");
            // Flush the prompt to appear before command
            let _ = io::stdout().flush();

            // Read line to compose program inpug
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;

            if line == "\n" {
                break;
            }

            input.push_str(&line);

            // Perform rep on whole available input
            match rep(&input) {
                Ok(output) => {
                    for el in output {
                        println!("{}", el);
                    }
                }
                Err(err) => {
                    if line == "\n" {
                        println!("ERROR: {}", err);
                    } else {
                        continue;
                    }
                }
            }
            break;
        }
    }
}
