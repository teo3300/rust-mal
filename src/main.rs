// io lib to read input and print output
use std::io::{self, Write};
use std::env::args;

mod env;
mod eval;
mod printer;
mod reader;
mod types;
use env::{Env,env_init};
use std::fs::File;
use std::io::{BufReader, BufRead};

mod step4_if_fn_do;
use step4_if_fn_do::rep;

use crate::types::Severity;

fn load_file(filename: &str, env: &Env) -> io::Result<()> {

    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut last: Result<(),()> = Ok(());

    let mut input = String::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                // Read line to compose program input
                input.push_str(&line);

                if input == ""{continue;}

                match rep(&input, env) {
                    Ok(_) => {
                        last = Ok(());
                        input = String::new()},
                    Err((err, Severity::Unrecoverable)) => {
                        last = Ok(());
                        println!("; Error @ {}", err);
                    },
                    _ => {last = Err(())}
                }
            },
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
    match last {
        Err(()) => println!("; ERROR parsing: '{}'\n;   the environment is in an unknown state", filename),
        _ => {}
    }
    Ok(())
}

fn main() {

    let reply_env = env_init();

    // setup env
    let args: Vec<String> = args().collect();
    for filename in &args[1..] {
        let _ = load_file(filename, &reply_env);
    }

    let mut num = 0;

    loop {
        let mut input = String::new();
        loop {
            print!("user> ");
            // Flush the prompt to appear before command
            let _ = io::stdout().flush();

            // Read line to compose program input
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();

            input.push_str(&line);

            if input != "\n" {
                // Perform rep on whole available input
                match rep(&input, &reply_env) {
                    Ok(output) => println!("[{}]> {}", num, output),
                    Err((err, sev)) => {
                        if sev == Severity::Recoverable && line != "\n" {
                            continue
                        }
                        println!("; [{}]> Error @ {}", num, err);
                    }
                }
                num += 1;
            }
            break;
        }
    }
}
