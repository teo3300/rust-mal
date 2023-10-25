use crate::types::MalType::*;
use crate::types::{MalMap, MalRet, MalType};
use std::cell::RefCell;
use std::rc::Rc;

// This is the first time I implement a macro, and I'm copying it
// so I will comment this a LOT
macro_rules! env_init {
    ($outer:expr) => {
        // match any istance with no args
        {
            // the macro prevent the macro from disrupting the external code
            // this is the block of code that will substitute the macro
            env_new($outer)
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
            let map = env_init!($outer);
            $( // Do this for all elements of the arguments list
                env_set(&map, $key, &$val);
            )*
            // return the new map
            map
        }
    };
}

#[derive(Clone)]
pub struct EnvType {
    data: RefCell<MalMap>,
    outer: Option<Env>,
}

pub type Env = Rc<EnvType>;
// Following rust implementation, using shorthand to always pas Reference count

pub fn env_new(outer: Option<Env>) -> Env {
    Rc::new(EnvType {
        data: RefCell::new(MalMap::new()),
        outer,
    })
}

pub fn env_set(env: &Env, sym: &str, val: &MalType) {
    env.data.borrow_mut().insert(sym.to_string(), val.clone());
}

pub fn env_get(env: &Env, sym: &String) -> MalRet {
    match env.data.borrow().get(sym) {
        Some(val) => Ok(val.clone()),
        None => match env.outer.clone() {
            Some(outer) => env_get(&outer, sym),
            None => Err(format!("symbol {:?} not defined", sym)),
        },
    }
}

use crate::printer::prt;

pub fn env_binds(outer: Env, binds: &MalType, exprs: &[MalType]) -> Result<Env, String> {
    let env = env_new(Some(outer));
    match binds {
        List(binds) => {
            if binds.len() != exprs.len() {
                return Err("Env init with unmatched length".to_string());
            } // TODO: May be possible to leave this be and not set additional elements at all
            for (bind, expr) in binds.iter().zip(exprs.iter()) {
                match bind {
                    Sym(sym) => env_set(&env, sym, expr),
                    _ => {
                        return Err(format!(
                            "Initializing environment: {:?} is not a symbol",
                            prt(bind)
                        ))
                    }
                }
            }
            Ok(env)
        }
        _ => Err("init: first argument must be a list".to_string()),
    }
}

use crate::types::MalType::{Fun, Str};
use crate::types::{arithmetic_op, comparison_op};
use std::process::exit;

fn panic() -> MalRet {
    panic!("If this messagge occurs, something went terribly wrong")
}

pub fn env_init() -> Env {
    env_init!(None,
              "test" => Fun(|_| Ok(Str("This is a test function".to_string())), "Just a test function"),
              "quit" => Fun(|_| {exit(0)}, "Quits the program with success status (0)"),
              "help" => Fun(|_| {panic()}, "Gets information about the symbols"),
              "+"    => Fun(|a| arithmetic_op(0, |a, b| a + b, a), "Returns the sum of the arguments"),
              "-"    => Fun(|a| arithmetic_op(0, |a, b| a - b, a), "Returns the difference of the arguments"),
              "*"    => Fun(|a| arithmetic_op(1, |a, b| a * b, a), "Returns the product of the arguments"),
              "/"    => Fun(|a| arithmetic_op(1, |a, b| a / b, a), "Returns the division of the arguments"),
              "="    => Fun(|a| comparison_op(|a, b| a == b, a), "Returns true if the arguments are equals, 'nil' otherwise"),
              ">"    => Fun(|a| comparison_op(|a, b| a >  b, a), "Returns true if the arguments are in strictly descending order, 'nil' otherwise"),
              "<"    => Fun(|a| comparison_op(|a, b| a >  b, a), "Returns true if the arguments are in strictly ascending order, 'nil' otherwise"),
              ">="   => Fun(|a| comparison_op(|a, b| a >= b, a), "Returns true if the arguments are in descending order, 'nil' otherwise"),
              "<="   => Fun(|a| comparison_op(|a, b| a <= b, a), "Returns true if the arguments are in ascending order, 'nil' otherwise")
    )
}