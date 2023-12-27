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

use crate::parse_tools::read_file;
use crate::printer::pr_str;
use crate::reader::{read_str, Reader};
use crate::types::MalType::{Bool, Fun, Int, List, Nil, Str};
use crate::types::{mal_assert, mal_assert_eq, mal_comp, MalArgs};

pub fn ns_init() -> Env {
    let env_t = env_init!(None,
        "test"          => Fun(|_| Ok(Str("This is a test function".to_string())), "Just a test function"),
        "exit"          => Fun(mal_exit, "Quits the program with specified status"),
        "+"             => Fun(|a| arithmetic_op(0, |a, b| a + b, a), "Returns the sum of the arguments"),
        "-"             => Fun(|a| arithmetic_op(0, |a, b| a - b, a), "Returns the difference of the arguments"),
        "*"             => Fun(|a| arithmetic_op(1, |a, b| a * b, a), "Returns the product of the arguments"),
        "/"             => Fun(|a| arithmetic_op(1, |a, b| a / b, a), "Returns the division of the arguments"),
        ">"             => Fun(|a| comparison_op(|a, b| a >  b, a), "Returns true if the arguments are in strictly descending order, 'nil' otherwise"),
        "<"             => Fun(|a| comparison_op(|a, b| a <  b, a), "Returns true if the arguments are in strictly ascending order, 'nil' otherwise"),
        ">="            => Fun(|a| comparison_op(|a, b| a >= b, a), "Returns true if the arguments are in descending order, 'nil' otherwise"),
        "<="            => Fun(|a| comparison_op(|a, b| a <= b, a), "Returns true if the arguments are in ascending order, 'nil' otherwise"),
        "pr-str"        => Fun(|a| Ok(Str(a.iter().map(|i| pr_str(i, true)).collect::<Vec<String>>().join(" "))), "Print readably all arguments"),
        "str"           => Fun(|a| Ok(Str(a.iter().map(|i| pr_str(i, false)).collect::<Vec<String>>().join(""))), "Print non readably all arguments"),
        "prn"           => Fun(|a| {a.iter().for_each(|a| print!("{} ", pr_str(a, false))); Ok(Nil) }, "Print readably all the arguments"),
        "println"       => Fun(|a| {a.iter().for_each(|a| print!("{} ", pr_str(a, false))); println!(); Ok(Nil) }, "Print readably all the arguments"),
        "list"          => Fun(|a| Ok(List(MalArgs::new(a.to_vec()))), "Return the arguments as a list"),
        "list?"         => Fun(|a| Ok(Bool(a.iter().all(|el| matches!(el, List(_))))), "Return true if the first argument is a list, false otherwise"),
        "empty?"        => Fun(|a| Ok(Bool(car(a)?.if_list()?.is_empty())), "Return true if the first parameter is an empty list, false otherwise, returns an error if the element is not a list"),
        "count"         => Fun(|a| Ok(Int(car(a)?.if_list()?.len() as isize)), "Return the number of elements in the first argument"),
        "and"           => Fun(|a| Ok(Bool(a.iter().all(|a| !matches!(a, Nil | Bool(false))))), "Returns false if at least one of the arguments is 'false' or 'nil', true otherwise"),
        "or"            => Fun(|a| Ok(Bool(a.iter().any(|a| !matches!(a, Nil | Bool(false))))), "Returns false if all the arguments are 'false' or 'nil', true otherwise"),
        "xor"           => Fun(|a| Ok(Bool(a.iter().filter(|a| !matches!(a, Nil | Bool(false))).count() == 1)), "Returns true if one of the arguments is different from 'nil' or 'false', false otherwise"),
        // "not" defined inside mal itself
        // "not"        => Fun(|a| Ok(Bool(matches!(car(a)?, Nil | Bool(false)))), "Negate the first argument"),
        "="             => Fun(mal_comp, "Return true if the first two parameters are the same type and content, in case of lists propagate to all elements"),
        "assert"        => Fun(mal_assert, "Return an error if assertion fails"),
        "assert-eq"     => Fun(mal_assert_eq, "Return an error if arguments are not the same"),
        "read-string"   => Fun(|a| read_str(&Reader::new().push(car(a)?.if_string()?)), "Tokenize and read the first argument"),
        "slurp"         => Fun(|a| Ok(Str(read_file(car(a)?.if_string()?)?)), "Read a file and return the content as a string")
    );
    env_t
}
