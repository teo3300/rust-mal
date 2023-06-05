// Structure the main functions of the interpreter
//
// For now just act as an echo, note that each function should not modify the
// input, thus this can be referenced by the previous step without the need
// to allocate more memory

use crate::printer::pr_str;
use crate::reader::read_str;
use crate::types::MalType;

#[allow(non_snake_case)]
/// Read input and generate an ast
fn READ(input: &str) -> MalType {
    read_str(input)
}

#[allow(non_snake_case)]
/// Evaluate the generated ast
fn EVAL(ast: MalType) -> MalType {
    ast
}

#[allow(non_snake_case)]
/// Print out the result of the evaluation
fn PRINT(input: MalType) -> String {
    pr_str(&input)
}

pub fn rep(input: &str) -> String {
    let ast = READ(input);
    let out = EVAL(ast);
    PRINT(out /*&result*/)
}
