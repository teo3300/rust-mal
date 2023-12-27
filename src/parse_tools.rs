use crate::env::Env;
use crate::eval::eval;
use crate::reader::{read_str, Reader};
use crate::step6_file::rep;
use crate::types::{MalErr, MalRet, MalType::Nil};
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::exit;

pub fn load_core(env: &Env) {
    eval_str("(def! not (fn* (x) (if x nil true)))", env).unwrap();
    eval_str(
        "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \"\nnil)\")))))",
        env,
    )
    .unwrap();
}

fn eval_str(line: &str, env: &Env) -> MalRet {
    eval(&read_str(Reader::new().push(line))?, env.clone())
}

pub fn load_conf(work_env: &Env) {
    const CONFIG: &str = "config.mal";
    let home = match env::var("MAL_HOME") {
        Ok(s) => s,
        Err(_) => env::var("HOME").unwrap() + "/.config/mal",
    };
    let config = home + "/" + CONFIG;

    if Path::new(&config).exists() {
        if let Err(e) = load_file(&config, work_env) {
            eprintln!("{}", e.message())
        }
    }
}

pub fn read_file(filename: &str) -> Result<String, MalErr> {
    let mut file = File::open(filename).map_err(|_| {
        MalErr::unrecoverable(format!("Failed to open file '{}'", filename).as_str())
    })?;
    let mut content = String::new();

    file.read_to_string(&mut content).map_err(|_| {
        MalErr::unrecoverable(format!("Failed to read content of '{}'", filename).as_str())
    })?;

    Ok(content)
}

pub fn load_file(filename: &str, env: &Env) -> MalRet {
    let file_desc = File::open(filename);
    let file = match file_desc {
        Ok(file) => file,
        Err(_) => {
            return Err(MalErr::unrecoverable(
                format!("; WARNING: Unable to open file: {}", filename).as_str(),
            ));
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
                        Ok(output) => output.iter().for_each(|el| eprintln!("[{}]> {}", num, el)),
                        Err(error) => {
                            if error.is_recoverable() {
                                // && line != "\n" {
                                continue;
                            }
                            eprintln!("; [{}]> Error @ {}", num, error.message());
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
