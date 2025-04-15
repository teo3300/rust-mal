use crate::env::Env;
use crate::eval::eval;
use crate::reader::{read_str, Reader};
use crate::step6_file::rep;
use crate::types::{MalErr, MalRet, MalStr};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;

fn eval_str(line: &str, env: &Env) -> MalRet {
    eval(&read_str(Reader::new().push(line))?, env.clone())
}

pub fn set_home_path(env: &Env) {
    eval_str(
        "(if (env \"MAL_HOME\")
        (def! MAL_HOME (env \"MAL_HOME\")) 
        (def! MAL_HOME (str (env \"HOME\") \"/.config/mal\")))",
        env,
    )
    .unwrap();
}

pub fn print_banner(env: &Env) {
    let _ = eval_str("(prn BANNER)", env);
}

fn get_home_path(env: &Env) -> Result<String, MalErr> {
    Ok(eval_str("MAL_HOME", env)?.if_string()?.to_string())
}

pub fn load_home_file(filename: &str, env: &Env, warn: bool) {
    let full_filename = get_home_path(env).unwrap_or_else(|_| "".to_string()) + "/" + filename;

    if Path::new(&full_filename).exists() {
        if let Err(e) = load_file(&full_filename, env) {
            eprintln!("; reading \"{}\":", full_filename);
            eprintln!("{}", e.message());
        }
    } else if warn {
        eprintln!("; WARNING: file \"{}\" does not exist", full_filename);
    }
}

pub fn read_file(filename: &str) -> Result<MalStr, MalErr> {
    let mut file = File::open(filename).map_err(|_| {
        MalErr::unrecoverable(format!("Failed to open file '{}'", filename).as_str())
    })?;
    let mut content = String::new();

    file.read_to_string(&mut content).map_err(|_| {
        MalErr::unrecoverable(format!("Failed to read content of '{}'", filename).as_str())
    })?;

    Ok(content.into())
}

pub fn load_file(filename: &str, env: &Env) -> MalRet {
    eval_str(
        format!(
            "(eval (read-string (str \"(do \" (slurp \"{}\") \"\nnil)\")))",
            filename
        )
        .as_str(),
        env,
    )
} // WTF this is becoming ever less like rust and more like lisp, did I really completely skip the file reading?

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub fn pre_load(argv: &Vec<String>, env: &Env) {
    eval_str(format!("(def! *ARGV* '({}))", argv[1..].iter().map(|x| x.to_string() + " ").collect::<String>()).as_str(), &env).unwrap();
}

pub fn interactive(env: Env) {
    const HISTORY: &str = ".mal-history";
    let home = get_home_path(&env).unwrap();
    let history = home + "/" + HISTORY;
    eval_str(format!("(def! MAL_HISTORY \"{}\")", history).as_str(), &env).unwrap();

    // Using "Editor" instead of the standard I/O because I hate myself but not this much
    // TODO: remove unwrap and switch to a better error handling
    let mut rl = DefaultEditor::new().unwrap();
    if rl.load_history(&history).is_err() {
        eprintln!("; Failed to load history");
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
            let line = rl.readline("; mal> ");

            match line {
                Ok(line) => {
                    // TODO: should handle this in a different way
                    rl.add_history_entry(&line).unwrap_or_default();
                    rl.save_history(&history)
                        .unwrap_or_else(|e| eprintln!("; WARNING: saving history: {}", e));

                    parser.push(&line);

                    // Perform rep on whole available input
                    match rep(&parser, &env) {
                        Ok(output) => output.iter().for_each(|el| {
                            eprintln!("; [{}]> {}", num, el);
                            num += 1;
                        }),
                        Err(error) => {
                            if error.is_recoverable() {
                                // && line != "\n" {
                                continue;
                            }
                            eprintln!("; [{}]> Error @ {}", num, error.message());
                            num += 1
                        }
                    }
                    break;
                }
                Err(ReadlineError::Interrupted) => {
                    parser.clear();
                    continue;
                }
                Err(ReadlineError::Eof) => exit(0),
                Err(err) => {
                    eprint!("; Error reading lnie: {:?}", err);
                    break;
                }
            }
        }
    }
}
