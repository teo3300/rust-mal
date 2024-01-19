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

    pub fn label_type(&self) -> MalType {
        let mut labl = "ʞ:".to_string();
        labl.push_str(match self {
            M::Nil => "nil",
            M::Bool(_) => "bool",
            M::Int(_) => "int",
            M::Fun(_, _) | M::MalFun { .. } => "lambda",
            M::Key(_) => "key",
            M::Str(_) => "string",
            M::Sym(_) => "symbol",
            M::List(_) => "list",
            M::Vector(_) => "vector",
            M::Map(_) => "map",
        });
        M::Key(labl)
    }
}

use crate::types::MalType as M;

// That's a quite chonky function
fn mal_compare(args: (&MalType, &MalType)) -> bool {
    match (args.0, args.1) {
        (M::Nil, M::Nil) => true,
        (M::Bool(a), M::Bool(b)) => a == b,
        (M::Int(a), M::Int(b)) => a == b,
        (M::Key(a), M::Key(b)) | (M::Str(a), M::Str(b)) | (M::Sym(a), M::Sym(b)) => a == b,
        (M::List(a), M::List(b)) | (M::Vector(a), M::Vector(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(mal_compare)
        }
        _ => false,
    }
}

pub fn mal_equals(args: &[MalType]) -> MalRet {
    Ok(M::Bool(match args.len() {
        0 => true,
        _ => {
            let (car, cdr) = car_cdr(args)?;
            cdr.iter().all(|x| mal_compare((car, x)))
        }
    }))
}

pub fn mal_assert(args: &[MalType]) -> MalRet {
    if args.iter().any(|i| matches!(i, M::Nil | M::Bool(false))) {
        return Err(MalErr::unrecoverable("Assertion failed"));
    }
    Ok(M::Nil)
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
