use std::collections::HashMap;

// All Mal types should inherit from this
#[derive(Debug, Clone)]
pub enum MalType {
    List(MalArgs),
    Vector(MalArgs),
    Map(MalMap),
    Fun(fn(&[MalType]) -> MalRet),
    Sym(String),
    Key(String),
    Str(String),
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
pub type MalMap = HashMap<String, MalType>;
pub type MalRet = Result<MalType, MalErr>;

use MalType::{Key, Map, Str};

pub fn make_map(list: MalArgs) -> MalRet {
    if list.len() % 2 != 0 {
        return Err("Map length is odd: missing value".to_string());
    }

    let mut map = MalMap::new();

    for i in (0..list.len()).step_by(2) {
        match &list[i] {
            Key(k) | Str(k) => {
                let v = &list[i + 1];
                map.insert(k.to_string(), v.clone());
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

use MalType::Int;

fn if_number(val: &MalType) -> Result<isize, String> {
    match val {
        Int(val) => Ok(*val),
        _ => Err(format!("{:?} is not a number", val)),
    }
}

pub fn int_op(set: isize, f: fn(isize, isize) -> isize, args: &[MalType]) -> MalRet {
    if args.is_empty() {
        return Ok(Int(set));
    }

    let mut left = if_number(&args[0])?;
    if args.len() > 1 {
        let right = &args[1..];
        for el in right {
            left = f(left, if_number(el)?);
        }
    }

    Ok(Int(left))
}
