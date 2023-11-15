use std::cell::{Cell, RefCell};

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

// TODO: instead of panic on missing ")" try implementing a multi line parsing
// Status on return should always be The last element of the last opened lists
// (append to the "last" list) while traversing
impl Reader {
    pub fn new() -> Reader {
        Reader {
            tokens: RefCell::new(Vec::new()),
            ptr: Cell::new(0),
        }
    }

    pub fn push(&self, input: &str) {
        self.ptr.set(0);
        // reset the state of the parser and push the additional strings
        self.tokens.borrow_mut().append(&mut tokenize(input))
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

    /// Repeatedly calls `read_form` of the reader object until it finds a ")" token
    /// EOF -> Return an error (Dyck language error)
    /// Accumulates results into a MalList
    /// NOTE: `read_list` calls `read_form` -> enable recursion
    /// (lists can contains other lists)
    fn read_list(&self, terminator: &str) -> MalRet {
        self.next()?;

        let mut vector = MalArgs::new();

        while self.peek()? != terminator {
            vector.push(self.read_form()?)
        }
        self.next()?;

        match terminator {
            ")" => Ok(List(vector)),
            "]" => Ok(Vector(vector)),
            "}" => make_map(vector),
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
                    Ok(Int(tk.parse::<isize>().unwrap()))
                } else if tk.starts_with('\"') {
                    Ok(Str(unescape_str(tk)))
                } else if tk.starts_with(':') {
                    Ok(Key(format!("ʞ{}", tk)))
                } else {
                    Ok(Sym(tk.to_string()))
                }
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
            _ => self.read_atom(),
        }
    }

    pub fn ended(&self) -> bool {
        self.tokens.borrow().len() == self.ptr.get()
    }
}

/// Call `tokenize` on a string
/// Create anew Reader with the tokens
/// Call read_from with the reader instance
pub fn read_str(reader: &Reader) -> MalRet {
    reader.read_form()
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
