use crate::env::{car_cdr, Env};
use std::{collections::HashMap, rc::Rc};

// All Mal types should inherit from this
#[derive(Clone, Debug)]
pub enum MalType {
    List(MalArgs),
    Vector(MalArgs),
    Map(MalMap),
    Fun(fn(&[MalType]) -> MalRet, &'static str), // Used for base functions, implemented using the underlying language (rust)
    MalFun {
        eval: fn(ast: &MalType, env: Env) -> MalRet,
        params: Rc<MalType>,
        ast: Rc<MalType>,
        env: Env,
    }, // Used for functions defined within mal
    // Use Rc so I can now clone like there's no tomorrow
    Sym(String),
    Key(String),
    Str(String),
    Int(isize),
    Bool(bool),
    Nil,
}

impl MalType {
    pub fn if_number(&self) -> Result<isize, MalErr> {
        match self {
            Self::Int(val) => Ok(*val),
            _ => Err(MalErr::unrecoverable(
                format!("{:?} is not a number", prt(self)).as_str(),
            )),
        }
    }

    pub fn if_list(&self) -> Result<&[MalType], MalErr> {
        match self {
            Self::List(list) => Ok(list),
            _ => Err(MalErr::unrecoverable(
                format!("{:?} is not a list", prt(self)).as_str(),
            )),
        }
    }

    pub fn if_symbol(&self) -> Result<&str, MalErr> {
        match self {
            Self::Sym(sym) => Ok(sym),
            _ => Err(MalErr::unrecoverable(
                format!("{:?} is not a symbol", prt(self)).as_str(),
            )),
        }
    }
}

use crate::types::MalType as M;

fn mal_eq(a: &M, b: &M) -> MalRet {
    Ok(M::Bool(match (a, b) {
        (M::Nil, M::Nil) => true,
        (M::Bool(a), M::Bool(b)) => a == b,
        (M::Int(a), M::Int(b)) => a == b,
        (M::Key(a), M::Key(b)) | (M::Str(a), M::Str(b)) => a == b,
        (M::List(a), M::List(b)) | (M::Vector(a), M::Vector(b)) => a
            .iter()
            .zip(b.iter())
            .all(|(a, b)| matches!(mal_eq(a, b), Ok(M::Bool(true)))),
        _ => {
            return Err(MalErr::unrecoverable(
                "Comparison not implemented for 'Map', 'Fun', 'MalFun' and 'Sym'",
            ))
        }
    }))
}

pub fn mal_comp(args: &[MalType]) -> MalRet {
    let (car, cdr) = car_cdr(args)?;
    match cdr.len() {
        0 => Ok(M::Bool(true)),
        _ => mal_eq(car, &cdr[0]),
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Severity {
    Recoverable,
    Unrecoverable,
}

pub struct MalErr {
    message: String,
    severity: Severity,
}

impl MalErr {
    pub fn new(message: String, severity: Severity) -> Self {
        Self { message, severity }
    }

    pub fn message(&self) -> String {
        self.message.to_string()
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn is_recoverable(&self) -> bool {
        self.severity == Severity::Recoverable
    }

    pub fn recoverable(message: &str) -> Self {
        Self::new(message.to_owned(), Severity::Recoverable)
    }

    pub fn unrecoverable(message: &str) -> Self {
        Self::new(message.to_owned(), Severity::Unrecoverable)
    }
}

pub type MalArgs = Vec<MalType>;
pub type MalMap = HashMap<String, MalType>;
pub type MalRet = Result<MalType, MalErr>;

use crate::printer::prt;
use MalType::{Key, Map, Str};

pub fn make_map(list: MalArgs) -> MalRet {
    if list.len() % 2 != 0 {
        return Err(MalErr::unrecoverable("Map length is odd: missing value"));
    }

    let mut map = MalMap::new();

    for i in (0..list.len()).step_by(2) {
        match &list[i] {
            Key(k) | Str(k) => {
                let v = &list[i + 1];
                map.insert(k.to_string(), v.clone());
            }
            _ => {
                return Err(MalErr::unrecoverable(
                    format!("Map key not valid: {}", prt(&list[i])).as_str(),
                ))
            }
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
