// Specyfy components in "types"
use crate::types::*;
// By specifying enum variants it's possible to omit namespace
use crate::types::MalType::*;

use regex::Regex;

pub struct Reader {
    tokens: Vec<String>,
    ptr: usize,
}

// TODO: instead of panic on missing ")" try implementing a multi line parsing
// Status on return should always be The last element of the last opened lists
// (append to the "last" list) while traversing
impl Reader {
    fn new(tokens: Vec<String>) -> Reader {
        Reader { tokens, ptr: 0 }
    }

    // May be improved
    fn get_token(&self, i: usize) -> Result<String, MalErr> {
        match self.tokens.get(i) {
            Some(token) => Ok(token.to_string()),
            None => Err("Unexpected EOF".to_string()),
        }
    }

    /// Returns the token at the current position
    fn peek(&self) -> Result<String, MalErr> {
        self.get_token(self.ptr)
    }

    /// Returns the token at current position and increment current position
    fn next(&mut self) -> Result<String, MalErr> {
        self.ptr += 1;
        self.get_token(self.ptr - 1)
    }

    /// Repeatedly calls `read_form` of the reader object until it finds a ")" token
    /// EOF -> Return an error (Dyck language error)
    /// Accumulates results into a MalList
    /// NOTE: `read_list` calls `read_form` -> enable recursion
    /// (lists can contains other lists)
    fn read_list(&mut self, terminator: &str) -> MalRet {
        self.next()?;

        let mut vector = Vec::new();

        while self.peek()? != terminator {
            vector.push(self.read_form()?)
        }
        self.next()?;

        match terminator {
            ")" => Ok(List(vector)),
            "]" => Ok(Vector(vector)),
            "}" => make_map(vector),
            _ => Err("Unknown collection terminator".to_string()),
        }
    }

    /// Read atomic token and return appropriate scalar ()
    fn read_atom(&mut self) -> MalRet {
        match &self.next()?[..] {
            ")" | "]" | "}" => Err("Missing open parenthesis".to_string()),
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
    fn read_form(&mut self) -> MalRet {
        // String slice containing the whole string
        match &self.peek()?[..] {
            // Consume "(" and parse list
            "(" => self.read_list(")"),
            "[" => self.read_list("]"),
            "{" => self.read_list("}"),
            _ => self.read_atom(),
        }
    }
}

/// Call `tokenize` on a string
/// Create anew Reader with the tokens
/// Call read_from with the reader instance
pub fn read_str(input: &str) -> MalRet {
    Reader::new(tokenize(input)).read_form()
}

/// Read a string and return a list of tokens in it (following regex in README)
// Add error handling for strings that are not terminated
fn tokenize(input: &str) -> Vec<String> {
    Regex::new(r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*\n|[^\s\[\]{}('"`,;)]*)"###)
        .unwrap()
        .captures_iter(input)
        .map(|e| e[1].to_string())
        .filter(|e| !e.is_empty() || !e.starts_with(';'))
        .collect::<Vec<String>>()
}
