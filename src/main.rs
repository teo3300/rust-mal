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

        io::stdin().read_line(&mut input)?;

        println!("{}", rep(&input.replace("\n", " ")));
    }
}
