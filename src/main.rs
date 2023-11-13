// io lib to read input and print output
use std::env::args;
use std::io::{self, Write};

mod env;
mod eval;
mod printer;
mod reader;
mod types;
use env::{env_init, Env};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use types::MalErr;

mod step4_if_fn_do;
use step4_if_fn_do::rep;

fn load_file(filename: &str, env: &Env) -> io::Result<()> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut last: Result<Vec<String>, MalErr> = Ok(Vec::new());

    let comment_line = Regex::new(r#"^[\s]*;.*"#).unwrap();

    let mut input = String::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                // Read line to compose program inpu

                if line == "" || comment_line.is_match(&line) {
                    continue; // Don't even add it
                } else {
                    input.push_str(format!("{}\n", &line).as_str());
                }

                last = match rep(&input, env) {
                    Err(error) if error.is_recoverable() => Err(error),
                    tmp => {
                        input = String::new();
                        tmp.map_err(|error| {
                            println!("; Error @ {}", error.message());
                            error
                        })
                    }
                }
            }
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
    match last {
        Err(error) => println!(
            "; ERROR parsing: '{}'\n;   {}\n;   the environment is in an unknown state",
            filename,
            error.message()
        ),
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
                    Ok(output) => output.iter().for_each(|el| println!("[{}]> {}", num, el)),
                    Err(error) => {
                        if error.is_recoverable() && line != "\n" {
                            continue;
                        }
                        println!("; [{}]> Error @ {}", num, error.message());
                    }
                }
                num += 1;
            }
            break;
        }
    }
}
