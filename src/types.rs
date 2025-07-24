use crate::env::{car_cdr, Env};
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

pub type MalStr = Rc<str>;
pub type MalArgs = Rc<[MalType]>;
pub type MalMap = HashMap<MalStr, MalType>;
pub type MalRet = Result<MalType, MalErr>;

#[derive(Clone, Copy)]
pub struct Frac {
    num: isize,
    den: usize,
}

impl Add for Frac {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            num: self.num * other.den as isize + other.num * self.den as isize,
            den: self.den * other.den,
        }
    }
}

impl Sub for Frac {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            num: self.num * other.den as isize - other.num * self.den as isize,
            den: self.den * other.den,
        }
    }
}

impl Mul for Frac {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self {
            num: self.num * other.num,
            den: self.den * other.den,
        }
    }
}

impl Div for Frac {
    type Output = Self;
    fn div(self, other: Frac) -> Self {
        let other_sign = other.num.signum();
        Self {
            num: self.num * other.den as isize * other_sign,
            den: self.den * other.num.unsigned_abs(),
        }
    }
}

impl PartialOrd for Frac {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.num * other.den as isize).cmp(&(other.num * self.den as isize)))
    }
}

impl PartialEq for Frac {
    fn eq(&self, other: &Self) -> bool {
        (!((self.num < 0) ^ (other.num < 0)))
            && self.num.unsigned_abs() * other.den == other.num.unsigned_abs() * self.den
    }
}

use std::fmt;

impl fmt::Display for Frac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl Frac {
    //pub fn frac(num: isize, den: usize) -> Self {
    //    Self { num, den }
    //}

    pub fn num(num: isize) -> Self {
        Self { num, den: 1 }
    }

    pub fn exact_zero(&self) -> bool {
        self.num == 0
    }

    pub fn get_num(&self) -> isize {
        self.num
    }

    pub fn get_den(&self) -> usize {
        self.den
    }

    fn _gcd(&self) -> usize {
        let mut t: usize;
        let mut a = self.num.unsigned_abs();
        let mut b = self.den;
        while b > 0 {
            t = b;
            b = a % b;
            a = t;
        }
        a
    }

    // Ideally builtin functions ( + - * / ) can operate with the values in any
    // form without problems, occasional simplification is done for numeric
    // stability, ensure to simplify after insert and before using any funcition
    // (floor, ceil, etc.)
    pub fn simplify(&self) -> Frac {
        // Euclid's algorithm to reduce fraction
        // TODO: (decide if implementing this automathically once fraction
        //          numbers become bigger than specified)
        let gcd = self._gcd();
        Frac {
            num: self.num / gcd as isize,
            den: self.den / gcd,
        }
    }

    pub fn int(&self) -> isize {
        self.num / self.den as isize
    }

    // return Ok(Num(Frac::num(tk.parse::<isize>().unwrap())));

    pub fn from_str(tk: &str) -> Option<Self> {
        let frac = match tk.find("/") {
            Some(v) => {
                let num = match tk[0..v].parse::<isize>() {
                    Ok(v) => v,
                    Err(_) => return None,
                };
                let den = match tk[v + 1..tk.len()].parse::<usize>() {
                    Ok(v) => v,
                    Err(_) => return None,
                };
                Self { num, den }
            }
            None => {
                let num = match tk.parse::<isize>() {
                    Ok(v) => v,
                    Err(_) => return None,
                };
                Self::num(num)
            }
        };
        // Ensure that value is simplified before being inserted
        // otherwise
        // (/ 4 4)  results in 1/1
        // 4/4      results in 4/4
        // this breaks some functions (like ceil) and doesn't make much sense
        Some(frac.simplify())
    }
}

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
    Sym(MalStr),
    Key(MalStr),
    Str(MalStr),
    Ch(char),
    Num(Frac),
    Atom(Rc<RefCell<MalType>>),
    Nil,
    T,
}

impl Default for &MalType {
    fn default() -> Self {
        &MalType::Nil
    }
}

impl MalType {
    pub fn if_number(&self) -> Result<Frac, MalErr> {
        match self {
            Self::Num(val) => Ok(*val),
            _ => Err(MalErr::unrecoverable(
                format!("{:?} is not a number", prt(self)).as_str(),
            )),
        }
    }

    pub fn if_list(&self) -> Result<&[MalType], MalErr> {
        match self {
            Self::List(list) | Self::Vector(list) => Ok(list),
            _ => Err(MalErr::unrecoverable(
                format!("{:?} is not an iterable", prt(self)).as_str(),
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
        Key(("ʞ:".to_owned()
            + match self {
                M::Nil => "nil",
                M::T => "t",
                M::Num(_) => "number",
                M::Fun(_, _) | M::MalFun { .. } => "lambda",
                M::Key(_) => "key",
                M::Str(_) => "string",
                M::Sym(_) => "symbol",
                M::List(_) => "list",
                M::Vector(_) => "vector",
                M::Map(_) => "map",
                M::Atom(_) => "atom",
                M::Ch(_) => "char",
            })
        .into())
    }
}

use crate::types::MalType as M;

// That's a quite chonky function
fn mal_compare(args: (&MalType, &MalType)) -> bool {
    match (args.0, args.1) {
        (M::Nil, M::Nil) => true,
        (M::T, M::T) => true,
        (M::Num(a), M::Num(b)) => a == b,
        (M::Ch(a), M::Ch(b)) => a == b,
        (M::Key(a), M::Key(b)) | (M::Str(a), M::Str(b)) | (M::Sym(a), M::Sym(b)) => a == b,
        (M::List(a), M::List(b)) | (M::Vector(a), M::Vector(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(mal_compare)
        }
        _ => false,
    }
}

pub fn mal_equals(args: &[MalType]) -> MalRet {
    Ok(match args.len() {
        0 => M::T,
        _ => {
            let (car, cdr) = car_cdr(args)?;
            if cdr.iter().all(|x| mal_compare((car, x))) {
                M::T
            } else {
                M::Nil
            }
        }
    })
}

pub fn reset_bang(args: &[MalType]) -> MalRet {
    if args.len() < 2 {
        return Err(MalErr::unrecoverable("reset requires two arguments"));
    }
    let val = &args[1];
    match &args[0] {
        M::Atom(sym) => {
            *std::cell::RefCell::<_>::borrow_mut(sym) = val.clone();
            Ok(val.clone())
        }
        _ => Err(MalErr::unrecoverable(
            format!("{:?} is not an atom", prt(&args[1])).as_str(),
        )),
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
                map.insert(k.clone(), v.clone());
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
            .replace('\u{000D}', "\\r")
            .replace('\t', "\\t")
            .replace('\"', "\\\"")
    )
}

pub fn unescape_str(s: &str) -> String {
    String::from(&s[1..s.len() - 1])
        .replace("\\\\", "\\")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\\"", "\"")
}

////////////////////////////////////////////////////////////////////////////////
// Tests                                                                      //
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {

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
