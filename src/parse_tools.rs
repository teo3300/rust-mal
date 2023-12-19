use crate::env::Env;
use crate::reader::Reader;
use crate::step5_tco::rep;
use crate::types::{MalErr, MalRet, MalType::Nil};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn load_file(filename: &str, env: &Env) -> MalRet {
    let file_desc = File::open(filename);
    let file = match file_desc {
        Ok(file) => file,
        Err(_) => {
            println!("Unable to open file: '{}'", filename);
            exit(1)
        }
    };
    let reader = BufReader::new(file);
    let mut last: Result<Vec<String>, MalErr> = Ok(Vec::new());

    let comment_line = Regex::new(r"^[\s]*;.*").unwrap();

    let parser = Reader::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                // Read line to compose program inpu
                if line.is_empty() || comment_line.is_match(&line) {
                    last = Ok(Vec::new());
                    continue;
                } else {
                    parser.push(&line);
                }

                last = match rep(&parser, env) {
                    Err(error) if error.is_recoverable() => Err(error),
                    tmp => {
                        parser.clear();
                        Ok(tmp.map_err(|error| {
                            MalErr::unrecoverable(format!("; Error @ {}", error.message()).as_str())
                        })?)
                    }
                }
            }
            Err(err) => {
                return Err(MalErr::unrecoverable(
                    format!("Error reading line: {}", err).as_str(),
                ))
            }
        }
    }
    if let Err(error) = last {
        Err(MalErr::unrecoverable(
            format!(
                "; ERROR parsing: '{}'\n;   {}\n;   the environment is in an unknown state",
                filename,
                error.message()
            )
            .as_str(),
        ))
    } else {
        Ok(Nil)
    }
}

use std::io::Write;
use std::process::exit;

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
                    if error.is_recoverable() {
                        // && line != "\n" {
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
