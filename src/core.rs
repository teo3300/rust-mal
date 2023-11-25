use crate::env::{arithmetic_op, car, comparison_op, env_new, env_set, mal_exit, Env};

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

use crate::printer::prt;
use crate::types::MalType::{Bool, Fun, Int, List, Nil, Str};
use crate::types::{mal_assert, mal_comp};

pub fn ns_init() -> Env {
    env_init!(None,
        "test"      => Fun(|_| Ok(Str("This is a test function".to_string())), "Just a test function"),
        "exit"      => Fun(mal_exit, "Quits the program with specified status"),
        "+"         => Fun(|a| arithmetic_op(0, |a, b| a + b, a), "Returns the sum of the arguments"),
        "-"         => Fun(|a| arithmetic_op(0, |a, b| a - b, a), "Returns the difference of the arguments"),
        "*"         => Fun(|a| arithmetic_op(1, |a, b| a * b, a), "Returns the product of the arguments"),
        "/"         => Fun(|a| arithmetic_op(1, |a, b| a / b, a), "Returns the division of the arguments"),
        ">"         => Fun(|a| comparison_op(|a, b| a >  b, a), "Returns true if the arguments are in strictly descending order, 'nil' otherwise"),
        "<"         => Fun(|a| comparison_op(|a, b| a >  b, a), "Returns true if the arguments are in strictly ascending order, 'nil' otherwise"),
        ">="        => Fun(|a| comparison_op(|a, b| a >= b, a), "Returns true if the arguments are in descending order, 'nil' otherwise"),
        "<="        => Fun(|a| comparison_op(|a, b| a <= b, a), "Returns true if the arguments are in ascending order, 'nil' otherwise"),
        "prn"       => Fun(|a| { println!("{} ", prt(car(a)?)); Ok(Nil) }, "Print readably all the arguments passed to it"),
        "list"      => Fun(|a| Ok(List(a.to_vec())), "Return the arguments as a list"),
        "list?"     => Fun(|a| Ok(Bool(matches!(car(a)?, List(_)))), "Return true if the first argument is a list, false otherwise"),
        "empty?"    => Fun(|a| Ok(Bool(car(a)?.if_list()?.is_empty())), "Return true if the first parameter is an empty list, false otherwise, returns an error if the element is not a list"),
        "count"     => Fun(|a| Ok(Int(car(a)?.if_list()?.len() as isize)), "Return the number of elements in the first argument"),
        "="         => Fun(mal_comp, "Return true if the first two parameters are the same type and content, in case of lists propagate to all elements"),
        "assert"    => Fun(mal_assert, "Panic if one of the argument is false")
    )
}
