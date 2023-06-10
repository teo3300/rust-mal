use std::collections::HashMap;

use crate::types::{KeyType, MalRet, MalType};

pub struct Env {
    map: HashMap<String, MalType>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            map: HashMap::new(),
        }
    }

    pub fn solve(&self, sym: KeyType) -> MalRet {
        let v = sym.val;
        match self.map.get(&v) {
            Some(val) => Ok(val.clone()),
            None => Err(format!("symbol {:?} not defined", v)),
        }
    }
}
