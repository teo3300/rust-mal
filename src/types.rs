// TODO: use enums for MalTypes

use std::collections::HashMap;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum KeyVar {
    Key,
    Sym,
    Str,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct KeyType {
    pub val: String,
    var: KeyVar,
}

pub fn map_key(var: KeyVar, val: &str) -> KeyType {
    KeyType {
        val: val.to_string(),
        var,
    }
}

// All Mal types should inherit from this
#[derive(Debug, Clone)]
pub enum MalType {
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Map(HashMap<KeyType, MalType>),
    Sym(KeyType),
    Key(KeyType),
    Str(KeyType),
    Int(isize),
    Bool(bool),
    Nil,
}

// Stolen, but this way it's easier to handle errors

/*
#[derive(Debug)]
pub enum MalErr {
    Str(String), // Messages to the user
                 // Val(MalType),
                 // Messages to the program
} TEMP TEMP  */
pub type MalErr = String;

pub type MalArgs = Vec<MalType>;
pub type MalRet = Result<MalType, MalErr>;

use MalType::{Key, Map, Str, Sym};

pub fn make_map(list: MalArgs) -> MalRet {
    if list.len() % 2 != 0 {
        return Err("Map length is odd: missing value".to_string());
    }

    let mut map = HashMap::new();

    for i in (0..list.len()).step_by(2) {
        match &list[i] {
            Sym(k) | Key(k) | Str(k) => {
                let v = list[i + 1].clone();
                map.insert(k.clone(), v);
            }
            _ => return Err(format!("Map key not valid: {:?}", list[i])),
        }
    }
    Ok(Map(map))
}

pub fn escape_str(s: &str) -> String {
    format!(
        "\"{}\"",
        String::from(s)
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\"', "\\\"")
    )
}

pub fn unescape_str(s: &str) -> String {
    String::from(&s[1..s.len() - 1])
        .replace("\\\\", "\\")
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
}
