use crate::types::MalType::*;
use crate::types::{MalArgs, MalMap, MalRet, MalType};

// This is the first time I implement a macro, and I'm copying it
// so I will comment this a LOT
macro_rules! env_init {
    ($outer:expr) => {
        // match any istance with no args
        {
            // the macro prevent the macro from disrupting the external code
            // this is the block of code that will substitute the macro
            Env::new($outer)
            // returns an empty map
        }
    };
    ($outer:expr, $($key:expr => $val:expr),*) => {
        // Only if previous statements did not match,
        // note that the expression with fat arrow is arbitrary,
        // could have been slim arrow, comma or any other
        // recognizable structure
        {
            // create an empty map
            let mut map = env_init!($outer);
            $( // Do this for all elements of the arguments list
                map.set($key, &$val);
            )*
            // return the new map
            map
        }
    };
}

#[derive(Clone)]
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

    pub fn init(&mut self, binds: MalArgs, exprs: MalArgs) -> Result<&mut Self, String> {
        if binds.len() != exprs.len() {
            return Err("Env init with unmatched length".to_string());
        } // TODO: May be possible to leave this be and not set additional elements at all
        for (bind, expr) in binds.iter().zip(exprs.iter()) {
            match bind {
                Sym(sym) => self.set(sym, expr),
                _ => {
                    return Err(format!(
                        "Initializing environment: {:?} is not a symbol",
                        bind
                    ))
                }
            }
        }
        Ok(self)
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

use crate::types::MalType::{Fun, Str};
use crate::types::{arithmetic_op, comparison_op};
use std::process::exit;

pub fn env_init() -> Env {
    env_init!(None,
              "test" => Fun(|_| Ok(Str("This is a test function".to_string()))),
              "quit" => Fun(|_| {println!("Bye!"); exit(0)}),
              "+"    => Fun(|a| arithmetic_op(0, |a, b| a + b, a)),
              "-"    => Fun(|a| arithmetic_op(0, |a, b| a - b, a)),
              "*"    => Fun(|a| arithmetic_op(1, |a, b| a * b, a)),
              "/"    => Fun(|a| arithmetic_op(1, |a, b| a / b, a)),
              "="    => Fun(|a| comparison_op(|a, b| a == b, a)),
              ">"    => Fun(|a| comparison_op(|a, b| a >  b, a)),
              "<"    => Fun(|a| comparison_op(|a, b| a >  b, a)),
              ">="   => Fun(|a| comparison_op(|a, b| a >= b, a)),
              "<="   => Fun(|a| comparison_op(|a, b| a <= b, a))
    )
}
