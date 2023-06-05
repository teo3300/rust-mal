// TODO: use enums for MalTypes

// All Mal types should inherit from this
#[derive(Debug)]
pub enum MalType {
    List(Vec<MalType>),
    Symbol(String),
    Integer(i32),
}
