use crate::env::Env;
use std::collections::HashMap;

// All Mal types should inherit from this
#[derive(Clone)]
pub enum MalType {
    List(MalArgs),
    Vector(MalArgs),
    Map(MalMap),
    Fun(fn(&[MalType]) -> MalRet, &'static str), // Used for base functions, implemented using the underlying language (rust)
    MalFun {
        eval: fn(ast: &MalType, env: Env) -> MalRet,
        params: Box<MalType>,
        ast: Box<MalType>,
        env: Env,
    }, // Used for functions defined within mal
    Sym(String),
    Key(String),
    Str(String),
    Int(isize),
    Bool(bool),
    Nil,
}

#[derive(PartialEq)]
pub enum Severity {
    Recoverable,
    Unrecoverable
}

pub type MalErr = String;
pub type MalArgs = Vec<MalType>;
pub type MalMap = HashMap<String, MalType>;
pub type MalRet = Result<MalType, MalErr>;

use MalType::{Key, Map, Str};
use crate::printer::prt;

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
            _ => return Err(format!("Map key not valid: {}", prt(&list[i]))),
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

/// Extract the car and cdr from a list
pub fn car_cdr(list: &[MalType]) -> (&MalType, &[MalType]) {
    (
        &list[0],
        if list.len() > 1 {
            &list[1..]
        } else {
            &list[0..0]
        },
    )
}

use MalType::Int;

fn if_number(val: &MalType) -> Result<isize, String> {
    match val {
        Int(val) => Ok(*val),
        _ => Err(format!("{:?} is not a number", prt(&val))),
    }
}

pub fn arithmetic_op(set: isize, f: fn(isize, isize) -> isize, args: &[MalType]) -> MalRet {
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

use MalType::{Bool, Nil};

pub fn comparison_op(f: fn(isize, isize) -> bool, args: &[MalType]) -> MalRet {
    match args.len() {
        0 => Err("Comparison requires at least 1 argument".to_string()),
        _ => {
            let (left, rights) = car_cdr(args);
            let mut left = if_number(left)?;
            for right in rights {
                let right = if_number(right)?;
                if !f(left, right) {
                    return Ok(Nil);
                }
                left = right;
            }
            Ok(Bool(true))
        }
    }
}
