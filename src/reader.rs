use std::cell::{Cell, RefCell};
use std::rc::Rc;

// Specyfy components in "types"
use crate::types::*;
// By specifying enum variants it's possible to omit namespace
use crate::types::MalType::*;

use regex::Regex;

pub struct Reader {
    tokens: RefCell<Vec<String>>,
    ptr: Cell<usize>,
}

type Tokens = Vec<String>;

// DONE: instead of panic on missing ")" try implementing a multi line parsing
// Status on return should always be The last element of the last opened lists
// (append to the "last" list) while traversing
impl Reader {
    pub fn new() -> Reader {
        Reader {
            tokens: RefCell::new(Vec::new()),
            ptr: Cell::new(0),
        }
    }

    pub fn push(&self, input: &str) -> &Self {
        self.ptr.set(0);
        // reset the state of the parser and push the additional strings
        self.tokens.borrow_mut().append(&mut tokenize(input));
        self
    }

    pub fn clear(&self) {
        self.ptr.set(0);
        *self.tokens.borrow_mut() = Vec::new();
    }

    // May be improved
    fn get_token(&self, i: usize) -> Result<String, MalErr> {
        self.tokens
            .borrow()
            .get(i)
            .ok_or(MalErr::recoverable("Unexpected EOF"))
            .cloned()
    }

    /// Returns the token at the current position
    fn peek(&self) -> Result<String, MalErr> {
        self.get_token(self.ptr.get())
    }

    /// Returns the token at current position and increment current position
    fn next(&self) -> Result<String, MalErr> {
        self.ptr.set(self.ptr.get() + 1);
        self.get_token(self.ptr.get() - 1)
    }

    /// Returns true if the reader has been consumed entirely
    pub fn ended(&self) -> bool {
        self.tokens.borrow().len() == self.ptr.get()
    }

    /// Repeatedly calls `read_form` of the reader object until it finds a ")" token
    /// EOF -> Return an error (Dyck language error)
    /// Accumulates results into a MalList
    /// NOTE: `read_list` calls `read_form` -> enable recursion
    /// (lists can contains other lists)
    fn read_list(&self, terminator: &str) -> MalRet {
        self.next()?;

        let mut vector = Vec::new();

        while self.peek()? != terminator {
            vector.push(self.read_form()?)
        }
        self.next()?;

        match terminator {
            ")" => Ok(List(MalArgs::new(vector))),
            "]" => Ok(Vector(MalArgs::new(vector))),
            "}" => make_map(MalArgs::new(vector)),
            t => Err(MalErr::unrecoverable(
                format!("Unknown collection terminator: {}", t).as_str(),
            )),
        }
    }

    /// Read atomic token and return appropriate scalar ()
    fn read_atom(&self) -> MalRet {
        match &self.next()?[..] {
            ")" | "]" | "}" => Err(MalErr::unrecoverable("Missing open parenthesis")),
            "false" => Ok(Bool(false)),
            "true" => Ok(Bool(true)),
            "nil" => Ok(Nil),
            tk => {
                if Regex::new(r"^-?[0-9]+$").unwrap().is_match(tk) {
                    return Ok(Int(tk.parse::<isize>().unwrap()));
                } else if tk.starts_with('\"') {
                    if tk.len() > 1 && tk.ends_with('\"') {
                        return Ok(Str(unescape_str(tk)));
                    }
                    return Err(MalErr::unrecoverable(
                        "End of line reached without closing string",
                    ));
                } else if tk.starts_with(':') {
                    return Ok(Key(format!("ʞ{}", tk)));
                }
                Ok(Sym(tk.to_string()))
            }
        }
    }

    /// Peek at the first token in reader
    ///
    /// Switch on the first character
    /// "(" -> call `read_list`
    /// otherwise  -> call `read_atom`
    fn read_form(&self) -> MalRet {
        // String slice containing the whole string
        match &self.peek()?[..] {
            // Consume "(" and parse list
            "(" => self.read_list(")"),
            "[" => self.read_list("]"),
            "{" => self.read_list("}"),
            // Ugly quote transformation for quote expansion
            "'" => {
                self.next()?;
                Ok(List(Rc::new(vec![
                    MalType::Sym("quote".to_string()),
                    self.read_form()?,
                ])))
            }
            _ => self.read_atom(),
        }
    }
}

/// Call `tokenize` on a string
/// Create anew Reader with the tokens
/// Call read_from with the reader instance
pub fn read_str(reader: &Reader) -> MalRet {
    let mut ret = Nil;
    while !reader.ended() {
        ret = reader.read_form()?;
    }
    Ok(ret)
}

/// Read a string and return a list of tokens in it (following regex in README)
// Add error handling for strings that are not terminated
fn tokenize(input: &str) -> Tokens {
    let tokens =
        Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#)
            .unwrap()
            .captures_iter(input)
            .map(|e| e[1].to_string())
            .filter(|e| !(e.is_empty() || e.starts_with(';')))
            .collect::<Vec<String>>();
    tokens
}

////////////////////////////////////////////////////////////////////////////////
// Tests                                                                      //
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::{
        reader::read_str,
        types::{MalMap, MalType as M},
    };

    use super::{tokenize, Reader};

    fn reader_setup1() -> Reader {
        let r = Reader::new();
        r.push("()[]{} \"str\" :key sym 1 ; comment");
        return r;
    }

    #[test]
    fn init() {
        let r = Reader::new();
        assert_eq!(r.tokens.borrow().len(), 0);
        assert_eq!(r.ptr.get(), 0);
    }

    #[test]
    fn do_tokenize() {
        assert_eq!(
            tokenize("()[]{} \"str\" :key sym 1 ; comment"),
            vec!["(", ")", "[", "]", "{", "}", "\"str\"", ":key", "sym", "1"]
        );
    }

    #[test]
    fn push() {
        let r = reader_setup1();
        let mut tokens = Vec::new();
        r.tokens
            .borrow()
            .iter()
            .for_each(|i| tokens.push(i.clone()));
        assert_eq!(
            tokens,
            vec!["(", ")", "[", "]", "{", "}", "\"str\"", ":key", "sym", "1"]
        );
    }

    #[test]
    fn get_token() {
        let r = reader_setup1();
        assert!(matches!(r.get_token(0), Ok(i) if i == "("));
        assert!(matches!(r.get_token(9), Ok(i) if i == "1"));
        assert!(matches!(r.get_token(10), Err(e) if e.is_recoverable()));
    }

    #[test]
    fn get() {
        let r = reader_setup1();
        assert!(matches!(r.peek(), Ok(i) if i == "("));
        assert_eq!(r.ptr.get(), 0);
        assert!(matches!(r.next(), Ok(i) if i == "("));
        assert_eq!(r.ptr.get(), 1);
        assert!(matches!(r.next(), Ok(i) if i == ")"));
        assert_eq!(r.ptr.get(), 2);
    }

    #[test]
    fn clear() {
        let r = reader_setup1();
        r.clear();
        assert_eq!(r.tokens.borrow().len(), 0);
        assert_eq!(r.ptr.get(), 0);
    }

    #[test]
    fn ended() {
        let r = reader_setup1();
        assert!(!r.ended());
        for _ in r.tokens.borrow().iter() {
            assert!(matches!(r.next(), Ok(_)))
        }
        assert!(r.ended());
    }

    #[test]
    fn errors() {
        let r = Reader::new();
        // Correct throws error
        assert!(matches!(r.peek(), Err(e) if e.is_recoverable()));
        assert!(matches!(r.next(), Err(e) if e.is_recoverable()));
    }

    #[test]
    fn read_atom() {
        let r = Reader::new();
        r.push("nil 1 true a \"s\" :a ) ] }");
        assert!(matches!(r.read_atom(), Ok(x) if matches!(x, M::Nil)));
        assert!(matches!(r.read_atom(), Ok(x) if matches!(x, M::Int(1))));
        assert!(matches!(r.read_atom(), Ok(x) if matches!(x, M::Bool(true))));
        assert!(matches!(r.read_atom(), Ok(x) if matches!(x.clone(), M::Sym(v) if v == "a")));
        assert!(matches!(r.read_atom(), Ok(x) if matches!(x.clone(), M::Str(v) if v == "s")));
        assert!(matches!(r.read_atom(), Ok(x) if matches!(x.clone(), M::Key(v) if v == "ʞ:a")));
        assert!(matches!(r.read_atom(), Err(e) if !e.is_recoverable()));
        assert!(matches!(r.read_atom(), Err(e) if !e.is_recoverable()));
        assert!(matches!(r.read_atom(), Err(e) if !e.is_recoverable()));
    }

    #[test]
    fn _read_str() {
        let r = Reader::new();

        // Test list
        let expected = vec![1, 2, 12];
        r.push("(1 2 12)");
        assert!(matches!(
            read_str(&r), Ok(x)
            if matches!(x.clone(), M::List(list)
                if list.len() == expected.len()
                && list.iter().zip(expected)
                    .all(|(x, y)| matches!(x, M::Int(v) if (*v as isize) == y)))));
        r.clear();

        // Test vector
        let exp = vec![1, 2, 12];
        r.push("[1 2 12]");
        assert!(matches!(
            read_str(&r), Ok(x)
            if matches!(x.clone(), M::Vector(list)
                if list.len() == exp.len()
                && list.iter().zip(exp)
                    .all(|(x, y)| matches!(x, M::Int(v) if (*v as isize) == y)))));
        r.clear();

        // Test map
        r.push("{\"i\" 1 \"s\" \"str\" \"t\" true \"n\" nil :s :sym}");
        let t = match read_str(&r) {
            Ok(x) => match x {
                M::Map(x) => x,
                _ => {
                    assert!(false);
                    MalMap::new()
                }
            },
            _ => {
                assert!(false);
                MalMap::new()
            }
        };
        assert!(matches!(t.get("n"),   Some(x) if matches!(&x, M::Nil)));
        assert!(matches!(t.get("t"),   Some(x) if matches!(&x, M::Bool(v) if *v)));
        assert!(matches!(t.get("i"),   Some(x) if matches!(&x, M::Int(v) if *v == 1)));
        assert!(matches!(t.get("s"),   Some(x) if matches!(&x, M::Str(v) if v == "str")));
        assert!(matches!(t.get("ʞ:s"), Some(x) if matches!(&x, M::Key(v) if v == "ʞ:sym")));
    }
}
