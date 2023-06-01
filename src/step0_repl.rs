// Structure the main functions of the interpreter
//
// For now just act as an echo, note that each function should not modify the
// input, thus this can be referenced by the previous step without the need
// to allocate more memory

#[allow(non_snake_case)]
/// Read input and generate an ast
fn READ(input: &str) -> String {
    input.to_string()
}

#[allow(non_snake_case)]
/// Evaluate the generated ast
fn EVAL(input: &str) -> String {
    input.to_string()
}

#[allow(non_snake_case)]
/// Print out the result of the evaluation
fn PRINT(input: &str) -> String {
    input.to_string()
}

pub fn rep(input: &str) -> String {
    let ast = READ(input);
    let result = EVAL(&ast);
    PRINT(&result)
}
