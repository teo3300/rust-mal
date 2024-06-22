// Structure the main functions of the interpreter
//
// For now just act as an echo, note that each function should not modify the
// input, thus this can be referenced by the previous step without the need
// to allocate more memory

// FIXME: (?) multiple sentences per line, only last is kept

use crate::env::Env;
use crate::eval::eval;
use crate::printer::pr_str;
use crate::reader::{read_str, Reader};
use crate::types::{MalErr, MalRet, MalType};

#[allow(non_snake_case)]
/// Read input and generate an ast
fn READ(input: &Reader) -> MalRet {
    read_str(input).map_err(|err| MalErr::new(format!("READ: {}", err.message()), err.severity()))
}

#[allow(non_snake_case)]
/// Evaluate the generated ast
fn EVAL(ast: MalType, env: Env) -> MalRet {
    eval(&ast, env).map_err(|err| MalErr::new(format!("EVAL: {}", err.message()), err.severity()))
}

#[allow(non_snake_case)]
/// Print out the result of the evaluation
fn PRINT(output: MalType) -> String {
    pr_str(&output, true)
}

pub fn rep(reader: &Reader, env: &Env) -> Result<Vec<String>, MalErr> {
    let mut ret_str = Vec::new();
    while !reader.ended() {
        let ast = READ(reader)?;
        let out = EVAL(ast, env.clone())?;
        ret_str.push(PRINT(out));
    }
    Ok(ret_str)
}
