use std::env;

use crate::env::{car, env_new, env_set, mal_exit, num_op, Env};

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

use crate::parse_tools::read_file;
use crate::printer::pr_str;
use crate::reader::{read_str, Reader};
use crate::types::MalType::{Bool, Fun, Int, List, Nil, Str};
use crate::types::{mal_assert, mal_comp, MalArgs, MalErr};

pub fn ns_init() -> Env {
    env_init!(None,
        // That's it, you are all going to be simpler functions
        "exit"          => Fun(mal_exit, "Quits the program with specified status"),
        "-"             => Fun(|a| num_op(|a, b|  Int(a -  b), a), "Returns the difference of the arguments"),
        "*"             => Fun(|a| num_op(|a, b|  Int(a *  b), a), "Returns the product of the arguments"),
        "/"             => Fun(|a| num_op(|a, b|  Int(a /  b), a), "Returns the division of the arguments"),
        ">"             => Fun(|a| num_op(|a, b| Bool(a >  b), a), "Returns true if the first argument is strictly greater than the second one, nil otherwise"),
        "<"             => Fun(|a| num_op(|a, b| Bool(a <  b), a), "Returns true if the first argument is strictly smallerthan the second one, nil otherwise"),
        "pr-str"        => Fun(|a| Ok(Str(a.iter().map(|i| pr_str(i, true)).collect::<Vec<String>>().join(" "))), "Print readably all arguments"),
        "str"           => Fun(|a| Ok(Str(a.iter().map(|i| pr_str(i, false)).collect::<Vec<String>>().join(""))), "Print non readably all arguments"),
        "prn"           => Fun(|a| {a.iter().for_each(|a| print!("{} ", pr_str(a, false))); Ok(Nil) }, "Print readably all the arguments"),
        "println"       => Fun(|a| {a.iter().for_each(|a| print!("{} ", pr_str(a, false))); println!(); Ok(Nil) }, "Print readably all the arguments"),
        "list"          => Fun(|a| Ok(List(MalArgs::new(a.to_vec()))), "Return the arguments as a list"),
        "list?"         => Fun(|a| Ok(Bool(a.iter().all(|el| matches!(el, List(_))))), "Return true if the first argument is a list, false otherwise"),
        "count"         => Fun(|a| Ok(Int(car(a)?.if_list()?.len() as isize)), "Return the number of elements in the first argument"),
        "="             => Fun(mal_comp, "Return true if the first two parameters are the same type and content, in case of lists propagate to all elements (NOT IMPLEMENTED for 'Map', 'Fun' and 'MalFun')"),
        "assert"        => Fun(mal_assert, "Return an error if assertion fails"),
        "read-string"   => Fun(|a| read_str(Reader::new().push(car(a)?.if_string()?)).map_err(MalErr::severe), "Tokenize and read the first argument"),
        "slurp"         => Fun(|a| Ok(Str(read_file(car(a)?.if_string()?)?)), "Read a file and return the content as a string"),
        "env"           => Fun(|a| match env::var(car(a)?.if_string()?) {
            Ok(s) => Ok(Str(s)),
            _ => Ok(Nil),
        }, "Retrieve the specified environment variable, returns NIL if that variable does not exist")
    )
}
