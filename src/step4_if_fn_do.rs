// Structure the main functions of the interpreter
//
// For now just act as an echo, note that each function should not modify the
// input, thus this can be referenced by the previous step without the need
// to allocate more memory

use crate::env::Env;
use crate::eval::eval;
use crate::printer::pr_str;
use crate::reader::read_str;
use crate::types::{MalRet, MalType};

#[allow(non_snake_case)]
/// Read input and generate an ast
fn READ(input: &str) -> MalRet {
    read_str(input).map_err(|err| format!("READ: {}", err))
}

#[allow(non_snake_case)]
/// Evaluate the generated ast
fn EVAL(ast: MalType, env: Env) -> MalRet {
    eval(&ast, env).map_err(|err| format!("EVAL: {}", err))
}

#[allow(non_snake_case)]
/// Print out the result of the evaluation
fn PRINT(output: MalType) -> String {
    pr_str(&output, true)
}

use crate::types::Severity;

pub fn rep(input: &str, env: &Env) -> Result<String, (String, Severity)> {
    let ast = READ(input)
        .map_err(|err| (err, Severity::Recoverable))?;
    let out = EVAL(ast, env.clone())
        .map_err(|err| (err, Severity::Unrecoverable))?;
    Ok(PRINT(out))
}
