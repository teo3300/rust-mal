use std::collections::{BTreeMap, VecDeque};

use crate::types::MalType;

use regex::Regex;

pub struct Reader {
    tokens: VecDeque<String>,
}

// TODO: instead of panic on missing ")" try implementing a multi line parsing
// Status on return should always be The last element of the last opened lists
// (append to the "last" list) while traversing
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
    fn read_list(&mut self, terminator: &str) -> Vec<MalType> {
        std::iter::from_fn(|| match self.peek() {
            ")" | "]" | "}" => {
                if terminator != self.peek() {
                    panic!("Unexpected token: {}", self.peek())
                }
                self.next();
                None
            }
            _ => Some(self.read_form()),
        })
        .collect()
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
                ")" | "]" | "}" => panic!("Lone parenthesis {}", token),
                "false" => MalType::Bool(false),
                "true" => MalType::Bool(true),
                "nil" => MalType::Nil,
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
                MalType::List(self.read_list(")"))
            }
            "[" => {
                self.next();
                MalType::Vector(self.read_list("]"))
            }
            "{" => {
                self.next();
                // fallback to C mode for now 😎
                let list = self.read_list("}");
                if list.len() % 2 != 0 {
                    panic!("Missing Map element")
                }
                let mut map = BTreeMap::new();
                for i in (0..list.len()).step_by(2) {
                    map.insert(list[i].clone(), list[i + 1].clone());
                }
                MalType::Map(map)
            }
            // read atomically
            _ => self.read_atom(),
        }
    }
}

/// Call `tokenize` on a string
/// Create anew Reader with the tokens
/// Call read_from with the reader instance
/// TODO: catch errors
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
