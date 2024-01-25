use std::env;

use crate::env::{any_zero, arithmetic_op, car, comparison_op, env_new, env_set, mal_exit, Env};

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
use crate::types::MalType::{Fun, Int, List, Nil, Str};
use crate::types::{mal_assert, mal_equals, MalErr};

pub fn ns_init() -> Env {
    env_init!(None,
        // That's it, you are all going to be simpler functions
        "exit"          => Fun(mal_exit, "Quits the program with specified status"),
        "raise"         => Fun(|a| Err(MalErr::unrecoverable(car(a)?.if_string()?)), "Raise an unrecoverable error with the specified message"),
        // Ok, keep * and / here because computing basic math operators recursively is fun but not too convenient
        "+"             => Fun(|a| arithmetic_op(0, |a, b| a +  b, a), "Returns the sum of the arguments"),
        "-"             => Fun(|a| arithmetic_op(0, |a, b| a -  b, a), "Returns the difference of the arguments"),
        "*"             => Fun(|a| arithmetic_op(1, |a, b| a *  b, a), "Returns the product of the arguments"),
        "/"             => Fun(|a| arithmetic_op(1, |a, b| a /  b, any_zero(a)?), "Returns the quotient of the arguments (not checking for division by 0)"),
        "<"             => Fun(|a| comparison_op(   |a, b| a <  b, a), "Returns true if the first argument is strictly smaller than the second one, nil otherwise"),
        ">"             => Fun(|a| comparison_op(   |a, b| a >  b, a), "Returns true if the first argument is strictly greater than the second one, nil otherwise"),
        "<="            => Fun(|a| comparison_op(   |a, b| a <= b, a), "Returns true if the first argument is smaller than or equal to the second one, nil otherwise"),
        ">="            => Fun(|a| comparison_op(   |a, b| a >= b, a), "Returns true if the first argument is greater than or equal to the second one, nil otherwise"),
        "pr-str"        => Fun(|a| Ok(Str(a.iter().map(|i| pr_str(i, true)).collect::<Vec<String>>().join(" ").into())), "Print readably all arguments"),
        "str"           => Fun(|a| Ok(Str(a.iter().map(|i| pr_str(i, false)).collect::<Vec<String>>().join("").into())), "Print non readably all arguments"),
        "prn"           => Fun(|a| {a.iter().for_each(|a| print!("{} ", pr_str(a, false))); Ok(Nil) }, "Print readably all the arguments"),
        "println"       => Fun(|a| {a.iter().for_each(|a| print!("{} ", pr_str(a, false))); println!(); Ok(Nil) }, "Print readably all the arguments"),
        "list"          => Fun(|a| Ok(List(a.into())), "Return the arguments as a list"),
        "type"          => Fun(|a| Ok(car(a)?.label_type()), "Returns a label indicating the type of it's argument"),
        "count"         => Fun(|a| Ok(Int(car(a)?.if_list()?.len() as isize)), "Return the number of elements in the first argument"),
        "="             => Fun(mal_equals, "Return true if the first two parameters are the same type and content, in case of lists propagate to all elements (NOT IMPLEMENTED for 'Map', 'Fun' and 'MalFun')"),
        "assert"        => Fun(mal_assert, "Return an error if assertion fails"),
        "read-string"   => Fun(|a| read_str(Reader::new().push(car(a)?.if_string()?)).map_err(MalErr::severe), "Tokenize and read the first argument"),
        "slurp"         => Fun(|a| Ok(Str(read_file(car(a)?.if_string()?)?)), "Read a file and return the content as a string"),
        "env"           => Fun(|a| match env::var(car(a)?.if_string()?) {
            Ok(s) => Ok(Str(s.into())),
            _ => Ok(Nil),
        }, "Retrieve the specified environment variable, returns NIL if that variable does not exist")
    )
}
