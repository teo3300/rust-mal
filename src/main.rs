// io lib to read input and print output
use std::io::{self, Write};

mod printer;
mod reader;
mod types;

mod step1_read_print;
use step1_read_print::rep;

fn main() -> io::Result<()> {
    loop {
        print!("user> ");
        // Flush the prompt to appear before command
        let _ = io::stdout().flush();

        let mut input = String::new();
        loop {
            // Read line to compose program inpug
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;

            // Append line to input
            input.push_str(&line);

            // If there is nothing to evaluate skip rep
            if input == "\n" {
                continue;
            }

            // Perform rep on whole available input
            match rep(&input) {
                Ok(output) => println!("{}", output),
                Err((err, depth)) => {
                    if line == "\n" {
                        println!("ERROR: {}", err);
                    } else {
                        print!("user> {}", "  ".repeat(depth));
                        // Flush the prompt to appear before command
                        let _ = io::stdout().flush();
                        continue;
                    }
                }
            }
            break;
        }
    }
}
