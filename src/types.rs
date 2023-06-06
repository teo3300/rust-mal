// TODO: use enums for MalTypes

use std::collections::BTreeMap;

// All Mal types should inherit from this
#[derive(Debug, Ord, Eq, PartialEq, PartialOrd, Clone)]
pub enum MalType {
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    // HashMap cannot implement Hash
    Map(BTreeMap<MalType, MalType>),
    Symbol(String),
    Integer(i32),
    Bool(bool),
    Nil,
}
