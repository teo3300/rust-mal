// Structure the main functions of the interpreter
//
// For now just act as an echo, note that each function should not modify the
// input, thus this can be referenced by the previous step without the need
// to allocate more memory

use crate::printer::pr_str;
use crate::reader::read_str;
use crate::types::{MalErr, MalType};

#[allow(non_snake_case)]
/// Read input and generate an ast
fn READ(input: &str) -> Result<Vec<MalType>, MalErr> {
    match read_str(input) {
        Ok(ast) => Ok(ast),
        Err(err) => Err(format!("Unexpected error during READ: {}", err)),
    }
}

#[allow(non_snake_case)]
/// Evaluate the generated ast
fn EVAL(ast: Vec<MalType>) -> Vec<MalType> {
    ast
}

#[allow(non_snake_case)]
/// Print out the result of the evaluation
fn PRINT(input: Vec<MalType>) -> Vec<String> {
    let mut ret = Vec::new();
    for expr in input {
        ret.push(pr_str(&expr, true))
    }
    ret
}

pub fn rep(input: &str) -> Result<Vec<String>, MalErr> {
    let ast = READ(input)?;
    let out = EVAL(ast);
    Ok(PRINT(out))
}
