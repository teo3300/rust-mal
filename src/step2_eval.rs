// Structure the main functions of the interpreter
//
// For now just act as an echo, note that each function should not modify the
// input, thus this can be referenced by the previous step without the need
// to allocate more memory

use crate::envs::Env;
use crate::mal_core::eval;
use crate::printer::pr_str;
use crate::reader::read_str;
use crate::types::{MalRet, MalType};

#[allow(non_snake_case)]
/// Read input and generate an ast
fn READ(input: &str) -> MalRet {
    match read_str(input) {
        Ok(ast) => Ok(ast),
        Err(err) => Err(format!("@ READ: {}", err)),
    }
}

#[allow(non_snake_case)]
/// Evaluate the generated ast
fn EVAL(ast: MalType, env: &Env) -> MalRet {
    match eval(ast, env) {
        Ok(ast) => Ok(ast),
        Err(err) => Err(format!("@ EVAL: {}", err)),
    }
}

#[allow(non_snake_case)]
/// Print out the result of the evaluation
fn PRINT(output: MalType) -> String {
    pr_str(&output, true)
}

pub fn rep(input: &str, env: &Env) -> Result<String, String> {
    let ast = READ(input)?;
    let out = EVAL(ast, env)?;
    Ok(PRINT(out))
}
