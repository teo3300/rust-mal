use std::io::{self, BufRead, Write};
use std::{cell::RefCell, env, rc::Rc};

use crate::env::{
    any_zero, arithmetic_op, car, comparison_op, env_new, env_set, mal_boom, mal_car, mal_cdr,
    mal_cons, mal_exit, Env,
};

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
use crate::printer::{pr_str, prt};
use crate::reader::{read_str, Reader};
use crate::types::{mal_equals, reset_bang, MalErr};
use crate::types::{
    Frac,
    MalType::{Atom, Fun, List, Nil, Num, Str},
};

macro_rules! if_atom {
    ($val:expr) => {{
        match $val {
            Atom(a) => Ok(a.borrow().clone()),
            _ => Err(MalErr::unrecoverable(
                format!("{:?} is not an atom", prt($val)).as_str(),
            )),
        }
    }};
}

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
        "pr-str"        => Fun(|a| Ok(Str(a.iter().map(|i| pr_str(i, true)).collect::<Vec<String>>().join("").into())), "Print readably all arguments"),
        "str"           => Fun(|a| Ok(Str(a.iter().map(|i| pr_str(i, false)).collect::<Vec<String>>().join("").into())), "Print non readably all arguments"),
        "prn"           => Fun(|a| {a.iter().for_each(|a| print!("{}", pr_str(a, false))); let _ = io::stdout().flush(); Ok(Nil) }, "Print readably all the arguments"),
        "println"       => Fun(|a| {a.iter().for_each(|a| print!("{}", pr_str(a, false))); println!(); Ok(Nil) }, "Print readably all the arguments"),
        "list"          => Fun(|a| Ok(List(a.into())), "Return the arguments as a list"),
        "type"          => Fun(|a| Ok(car(a)?.label_type()), "Returns a label indicating the type of it's argument"),
        "count"         => Fun(|a| Ok(Num(Frac::num(car(a)?.if_list()?.len() as isize))), "Return the number of elements in the first argument"),
        "="             => Fun(mal_equals, "Return true if the first two parameters are the same type and content, in case of lists propagate to all elements (NOT IMPLEMENTED for 'Map', 'Fun' and 'MalFun')"),
        "car"           => Fun(|a| mal_car(car(a)?), "Returns the first element of the list, NIL if its empty"),
        "cdr"           => Fun(|a| mal_cdr(car(a)?), "Returns all the list but the first element"),
        // Number functions, still to decide how to handle
        "num"           => Fun(|a| Ok(Num(Frac::num(car(a)?.if_number()?.get_num()))), "Get numerator of the number"),
        "den"           => Fun(|a| Ok(Num(Frac::num(car(a)?.if_number()?.get_den() as isize))), "Get denominator of the number"),
        "floor"         => Fun(|a| Ok(Num(Frac::num(car(a)?.if_number()?.int()))), "Approximate the number to the closest smaller integer"),
        // A tribute to PHP's explode (PHP, a language I never used)
        "boom"          => Fun(mal_boom, "Split a string into a list of characters\n; BE CAREFUL WHEN USING"),
        "read-string"   => Fun(|a| read_str(Reader::new().push(car(a)?.if_string()?)).map_err(MalErr::severe), "Tokenize and read the first argument"),
        "read-line"     => Fun(|_| Ok(Str(io::stdin().lock().lines().next().unwrap().unwrap().into())), "Read a line from input and return its content"),
        "slurp"         => Fun(|a| Ok(Str(read_file(car(a)?.if_string()?)?)), "Read a file and return the content as a string"),
        "atom"          => Fun(|a| Ok(Atom(Rc::new(RefCell::new(car(a).unwrap_or_default().clone())))), "Return an atom pointing to the given arg"),
        "deref"         => Fun(|a| if_atom!(car(a)?), "Return the content of the atom argumet"),
        "reset!"        => Fun(reset_bang, "Change the value of the Atom (frist argument) to the second argument"),
        "cons"          => Fun(mal_cons, "Push to front if second element is a list"),
        "env"           => Fun(|a| match env::var(car(a)?.if_string()?) {
            Ok(s) => Ok(Str(s.into())),
            _ => Ok(Nil),
        }, "Retrieve the specified environment variable, returns NIL if that variable does not exist")
    )
}
