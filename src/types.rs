use crate::env::{car_cdr, Env};
use std::{collections::HashMap, rc::Rc};

// All Mal types should inherit from this
#[derive(Clone)]
pub enum MalType {
    List(MalArgs),
    Vector(MalArgs),
    Map(MalMap),
    Fun(fn(&[MalType]) -> MalRet, &'static str), // Used for base functions, implemented using the underlying language (rust)
    MalFun {
        // eval: fn(ast: &MalType, env: Env) -> MalRet,
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

    pub fn if_vec(&self) -> Result<&[MalType], MalErr> {
        match self {
            Self::Vector(list) => Ok(list),
            _ => Err(MalErr::unrecoverable(
                format!("{:?} is not a vector", prt(self)).as_str(),
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

    pub fn if_string(&self) -> Result<&str, MalErr> {
        match self {
            Self::Str(sym) => Ok(sym),
            _ => Err(MalErr::unrecoverable(
                format!("{:?} is not a string", prt(self)).as_str(),
            )),
        }
    }
}

use crate::types::MalType as M;

// That's a quite chonky function
fn mal_eq(a: &MalType, b: &[MalType]) -> MalRet {
    Ok(M::Bool(match a {
        M::Nil => b.iter().all(|el| matches!(el, M::Nil)),
        M::Bool(a) => b.iter().all(|el| matches!(el, M::Bool(b) if a == b)),
        M::Int(a) => b.iter().all(|el| matches!(el, M::Int(b)  if a == b)),
        M::Key(a) => b.iter().all(|el| matches!(el, M::Key(b)  if a == b)),
        M::Str(a) => b.iter().all(|el| matches!(el, M::Str(b)  if a == b)),
        M::List(a) => b.iter().all(|el| {
            matches!(el, M::List(b)
                if a.len() == b.len()
                && a.iter().zip(b.iter()).all(
                    |(a, b)| matches!(mal_eq(a, &[b.clone()]),
                        Ok(M::Bool(true)))))
        }),
        M::Vector(a) => b.iter().all(|el| {
            matches!(el, M::Vector(b)
                if a.len() == b.len()
                && a.iter().zip(b.iter()).all(
                    |(a, b)| matches!(mal_eq(a, &[b.clone()]),
                        Ok(M::Bool(true)))))
        }),
        _ => {
            return Err(MalErr::unrecoverable(
                "Comparison not implemented for 'Map', 'Fun', 'MalFun' and 'Sym'",
            ))
        }
    }))
}

pub fn mal_comp(args: &[MalType]) -> MalRet {
    match args.len() {
        0 => Ok(M::Bool(true)),
        _ => {
            let (car, cdr) = car_cdr(args)?;
            mal_eq(car, cdr)
        }
    }
}

pub fn mal_assert(args: &[MalType]) -> MalRet {
    if args.iter().any(|i| matches!(i, M::Nil | M::Bool(false))) {
        Err(MalErr::unrecoverable("Assertion failed"))
    } else {
        Ok(M::Nil)
    }
}

pub fn mal_assert_eq(args: &[MalType]) -> MalRet {
    let (car, cdr) = car_cdr(args)?;
    match mal_eq(car, cdr)? {
        M::Nil | M::Bool(false) => {
            let mut message = String::from("Assertion-eq failed: [");
            message.push_str(
                args.iter()
                    .map(prt)
                    .collect::<Vec<String>>()
                    .join(" ")
                    .as_str(),
            );
            message.push(']');
            Err(MalErr::unrecoverable(message.as_str()))
        }
        _ => Ok(M::Nil),
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Severity {
    Recoverable,
    Unrecoverable,
}

#[derive(Debug)]
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

    pub fn severe(mut self) -> Self {
        self.severity = Severity::Unrecoverable;
        self
    }

    pub fn recoverable(message: &str) -> Self {
        Self::new(message.to_owned(), Severity::Recoverable)
    }

    pub fn unrecoverable(message: &str) -> Self {
        Self::new(message.to_owned(), Severity::Unrecoverable)
    }
}

pub type MalArgs = Rc<Vec<MalType>>;
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

////////////////////////////////////////////////////////////////////////////////
// Tests                                                                      //
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::types::mal_assert;
    use crate::types::MalType as M;

    #[test]
    fn _mal_assert() {
        assert!(matches!(mal_assert(&[M::Nil]), Err(_)));
        assert!(matches!(mal_assert(&[M::Bool(false)]), Err(_)));
        assert!(matches!(mal_assert(&[M::Bool(true)]), Ok(_)));
        assert!(matches!(mal_assert(&[M::Int(1)]), Ok(_)));
    }

    #[test]
    fn _escape_str() {
        use crate::types::escape_str;
        assert_eq!(escape_str(""), "\"\""); // add quotations
        assert_eq!(escape_str("\\"), "\"\\\\\""); // escape "\"
        assert_eq!(escape_str("\n"), "\"\\n\""); // escape "\n"
        assert_eq!(escape_str("\""), "\"\\\"\""); // escape "\""
    }

    #[test]
    fn _unescape_str() {
        use crate::types::unescape_str;
        assert_eq!(unescape_str("\"\""), ""); // remove quotations
        assert_eq!(unescape_str("\"a\""), "a"); // remove quotations
        assert_eq!(unescape_str("\"\\\\\""), "\\"); // unescape "\"
        assert_eq!(unescape_str("\"\\n\""), "\n"); // unescape "\n"
        assert_eq!(unescape_str("\"\\\"\""), "\""); // unescape "\""
    }
}
