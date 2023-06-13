use crate::types::{MalMap, MalRet, MalType};

pub struct Env {
    data: MalMap,
    outer: Option<Box<Env>>,
}

impl Env {
    pub fn new(outer: Option<Box<Env>>) -> Self {
        Env {
            data: MalMap::new(),
            outer,
        }
    }

    pub fn set(&mut self, sym: &str, val: &MalType) {
        self.data.insert(sym.to_string(), val.clone());
    }

    pub fn get(&self, sym: &String) -> MalRet {
        match self.data.get(sym) {
            Some(val) => Ok(val.clone()),
            None => match &self.outer {
                Some(outer) => outer.get(sym),
                None => Err(format!("symbol {:?} not defined", sym)),
            },
        }
    }
}
