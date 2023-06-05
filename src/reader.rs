use std::collections::VecDeque;

use crate::types::MalType;

use regex::Regex;

pub struct Reader {
    tokens: VecDeque<String>,
}

const PAREN_ERROR: &str =
    "Looks like you reached a dead end, did you perhaps miss any \")\" or left some extra \"(\"?";

impl Reader {
    fn new(tokens: VecDeque<String>) -> Reader {
        Reader { tokens }
    }

    /// Returns the token at the current positioni
    fn peek(&self) -> &str {
        match self.tokens.get(0) {
            Some(token) => token,
            None => panic!("{}", PAREN_ERROR),
        }
    }

    /// Returns the token at current position and increment current position
    // TODO: PLEASE USE THE PEEK FUNCTION
    fn next(&mut self) -> String {
        match self.tokens.pop_front() {
            Some(token) => token,
            None => panic!("{}", PAREN_ERROR),
        }
    }

    /// Repeatedly calls `read_form` of the reader object until it finds a ")" token
    /// EOF -> Return an error (Dyck language error)
    /// Accumulates results into a MalList
    /// NOTE: `read_list` calls `read_form` -> enable recursion
    /// (lists can contains other lists)
    fn read_list(&mut self) -> MalType {
        MalType::List(
            // Iterate over the the list
            std::iter::from_fn(|| match self.peek() {
                // consume "(" and return
                ")" => {
                    self.next();
                    None
                }
                // Add read the token recursively
                _ => Some(self.read_form()),
            })
            // create vector to return
            .collect(),
        )
    }

    /// Read atomic token and return appropriate scalar ()
    fn read_atom(&mut self) -> MalType {
        let token = self.next();
        // parse the token as an integer
        match token.parse::<i32>() {
            // On success assign the value
            Ok(value) => MalType::Integer(value),
            // Otherwise assign the symbol
            Err(_) => match token.as_str() {
                ")" => panic!("Invalid token \")\""),
                _ => MalType::Symbol(token),
            },
        }
    }

    /// Peek at the first token in reader
    ///
    /// Switch on the first character
    /// "(" -> call `read_list`
    /// otherwise  -> call `read_atom`
    fn read_form(&mut self) -> MalType {
        match self.peek() {
            // Consume "(" and parse list
            "(" => {
                self.next();
                self.read_list()
            }
            // read atomically
            _ => self.read_atom(),
        }
    }
}

#[allow(dead_code)]
fn pretty_print(ast: &MalType, base: usize) {
    print!("{}", "│  ".repeat(base));
    match ast {
        MalType::Symbol(sym) => println!("Sym: {}", sym),
        MalType::Integer(val) => println!("Int: {}", val),
        MalType::List(vec) => {
            println!("List: ");
            for el in vec {
                pretty_print(el, base + 1);
            }
        }
    }
}

/// Call `tokenize` on a string
/// Create anew Reader with the tokens
/// Call read_from with the reader instance
pub fn read_str(input: &str) -> MalType {
    let ast = Reader::new(tokenize(input)).read_form();
    // pretty_print(&ast, 0);
    ast
}

/// Read a string and return a list of tokens in it (following regex in README)
// Add error handling for strings that are not terminated
fn tokenize(input: &str) -> VecDeque<String> {
    let mut tokens = VecDeque::new();

    let re =
        Regex::new(r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"###)
            .unwrap();
    for match_str in re.captures_iter(input) {
        if match_str[1].len() > 0 {
            tokens.push_back(match_str[1].to_string());
        }
    }

    tokens
}
