// Specyfy components in "types"
use crate::types::*;
// By specifying enum variants it's possible to omit namespace
use crate::types::MalType::*;

use regex::Regex;

pub struct Reader {
    tokens: Vec<String>,
    ptr: usize,
    depth: usize,
}

// TODO: instead of panic on missing ")" try implementing a multi line parsing
// Status on return should always be The last element of the last opened lists
// (append to the "last" list) while traversing
impl Reader {
    fn new(tokens: Vec<String>) -> Reader {
        Reader {
            tokens,
            ptr: 0,
            depth: 0,
        }
    }

    /// Returns the token at the current position
    fn peek(&self) -> Result<String, MalErr> {
        match self.tokens.get(self.ptr) {
            Some(token) => Ok(token.to_string()),
            None => Err("Unexpected EOF, Missing parenthesis?".to_string()),
        }
    }

    /// Returns the token at current position and increment current position
    // TODO: PLEASE USE THE PEEK FUNCTION
    fn next(&mut self) -> Result<String, MalErr> {
        self.ptr += 1;
        match self.tokens.get(self.ptr - 1) {
            Some(token) => Ok(token.to_string()),
            None => Err("Unexpected EOF, Missing parenthesis?".to_string()),
        }
    }

    /// Repeatedly calls `read_form` of the reader object until it finds a ")" token
    /// EOF -> Return an error (Dyck language error)
    /// Accumulates results into a MalList
    /// NOTE: `read_list` calls `read_form` -> enable recursion
    /// (lists can contains other lists)
    fn read_list(&mut self, terminator: &str) -> MalRet {
        self.depth += 1;

        self.next()?;

        let mut vector = Vec::new();
        loop {
            let token = self.peek()?;
            if token == terminator {
                break;
            }
            vector.push(self.read_form()?)
        }
        self.next()?;
        let ret = match terminator {
            ")" => Ok(List(vector)),
            "]" => Ok(Vector(vector)),
            "}" => make_map(vector),
            _ => Err(format!("Unknown collection terminator: {}", terminator)),
        };
        self.depth -= 1;
        ret
    }

    /// Read atomic token and return appropriate scalar ()
    fn read_atom(&mut self) -> MalRet {
        let token = self.next()?;
        let re_digits = Regex::new(r"^-?[0-9]+$").unwrap();
        match token.as_str() {
            ")" | "]" | "}" => Err(format!("Lone parenthesis {}", token)),
            "false" => Ok(Bool(false)),
            "true" => Ok(Bool(true)),
            "nil" => Ok(Nil),
            _ => {
                if re_digits.is_match(&token) {
                    Ok(Int(token.parse::<isize>().unwrap()))
                } else if token.starts_with('\"') {
                    if token.ends_with('\"') {
                        Ok(Str(unescape_str(&token)))
                    } else {
                        Err("Unterminated string, expected \"".to_string())
                    }
                } else if token.starts_with(':') {
                    Ok(Keyword(token))
                } else {
                    Ok(Symbol(token))
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
        let token = self.peek()?;
        // String slice containing the whole string
        match &token[..] {
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
/// TODO: catch errors
pub fn read_str(input: &str) -> Result<MalType, (MalErr, usize)> {
    let tokens = tokenize(input);
    match tokens.len() {
        0 => Ok(Nil),
        _ => {
            let mut reader = Reader::new(tokens);
            match reader.read_form() {
                Err(err) => Err((err, reader.depth)),
                Ok(any) => Ok(any),
            }
        }
    }
}

/// Read a string and return a list of tokens in it (following regex in README)
// Add error handling for strings that are not terminated
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();

    let re = Regex::new(
        r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*\n|[^\s\[\]{}('"`,;)]*)"###,
    )
    .unwrap();
    for match_str in re.captures_iter(input) {
        if !match_str[1].is_empty() {
            // Drop comments
            if match_str[1].starts_with(';') {
                continue;
            }
            tokens.push(match_str[1].to_string());
        }
    }

    tokens
}
