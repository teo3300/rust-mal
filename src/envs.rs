use std::collections::HashMap;

use crate::types::MalType::{Fun, Str};
use crate::types::{int_op, MalArgs, MalRet, MalType};

pub struct Env {
    map: HashMap<String, MalType>,
}

impl Env {
    pub fn new() -> Self {
        let mut env = Env {
            map: HashMap::new(),
        };
        env.init();
        env
    }

    pub fn solve(&self, sym: String) -> MalRet {
        match self.map.get(&sym) {
            Some(val) => Ok(val.clone()),
            None => Err(format!("symbol {:?} not defined", sym)),
        }
    }

    fn define(&mut self, sym: &str, f: fn(MalArgs) -> MalRet) {
        self.map.insert(sym.to_string(), Fun(f));
    }

    fn init(&mut self) {
        self.define("test", |_| Ok(Str("This is a test function".to_string())));
        self.define("+", |args| int_op(0, |a, b| a + b, args));
        self.define("-", |args| int_op(0, |a, b| a - b, args));
        self.define("*", |args| int_op(1, |a, b| a * b, args));
        self.define("/", |args| int_op(1, |a, b| a / b, args));
    }
}
