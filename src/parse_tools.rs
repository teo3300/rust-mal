use crate::env::Env;
use crate::reader::Reader;
use crate::step5_tco::rep;
use crate::types::{MalErr, MalRet, MalType::Nil};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::exit;

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

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub fn interactive(env: Env) {
    const HISTORY_PATH: &str = ".mal-history";

    // Using "Editor" instead of the standard I/O because I hate myself but not this much
    // TODO: remove unwrap and switch to a better error handling
    let mut rl = DefaultEditor::new().unwrap();
    if rl.load_history(HISTORY_PATH).is_err() {
        eprintln!("Failed to load history");
    }

    let mut num = 0;
    let parser = Reader::new();
    loop {
        parser.clear();
        loop {
            // // Old reader
            // print!("user> ");
            // // Flush the prompt to appear before command
            // let _ = io::stdout().flush();

            // // Read line to compose program input
            // let mut line = String::new();
            // io::stdin().read_line(&mut line).unwrap();
            let line = rl.readline("user> ");

            match line {
                Ok(line) => {
                    // TODO: should handle this in a different way
                    rl.add_history_entry(&line).unwrap();
                    rl.save_history(HISTORY_PATH).unwrap();

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
                Err(ReadlineError::Interrupted) => {
                    parser.clear();
                    eprintln!("; ... Interrupted");
                    continue;
                }
                Err(ReadlineError::Eof) => exit(0),
                Err(err) => {
                    eprint!("Error reading lnie: {:?}", err);
                    break;
                }
            }
        }
    }
}
