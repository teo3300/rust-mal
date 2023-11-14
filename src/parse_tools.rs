use crate::env::Env;
use crate::reader::Reader;
use crate::step4_if_fn_do::rep;
use crate::types::MalErr;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn load_file(filename: &str, env: &Env) -> io::Result<()> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut last: Result<Vec<String>, MalErr> = Ok(Vec::new());

    let comment_line = Regex::new(r#"^[\s]*;.*"#).unwrap();

    let parser = Reader::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                // Read line to compose program inpu

                if line == "" || comment_line.is_match(&line) {
                    continue; // Don't even add it
                } else {
                    parser.push(&line);
                }

                last = match rep(&parser, env) {
                    Err(error) if error.is_recoverable() => {
                        Err(error)
                    }
                    tmp => {
                        parser.clear();
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

use std::io::Write;

pub fn interactive(env: Env) {
    let mut num = 0;
    let parser = Reader::new();
    loop {
        parser.clear();
        loop {
            print!("user> ");
            // Flush the prompt to appear before command
            let _ = io::stdout().flush();

            // Read line to compose program input
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();

            parser.push(&line);

            // Perform rep on whole available input
            match rep(&parser, &env) {
                Ok(output) => output.iter().for_each(|el| println!("[{}]> {}", num, el)),
                Err(error) => {
                    if error.is_recoverable() {// && line != "\n" {
                        continue;
                    }
                    println!("; [{}]> Error @ {}", num, error.message());
                }
            }
            num += 1;
            break;
        }
    }
}