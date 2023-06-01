// io lib to read input and print output
use std::io::{self, Write};

mod step0_repl;
use step0_repl::rep;

fn main() -> io::Result<()> {
    loop {
        print!("user> ");
        // Flush the prompt to appear before command
        let _ = io::stdout().flush();

        let mut input = String::new();

        io::stdin().read_line(&mut input)?;

        print!("{}", rep(&input));
    }
}
