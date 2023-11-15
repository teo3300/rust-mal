// This file should contain all the necessary function to define builtin functions

use crate::env::env_binds;
use crate::printer::prt;
use crate::types::MalType::{Fun, List, MalFun};
use crate::types::{MalErr, MalRet, MalType};

use MalType::Int;

pub fn scream() -> MalRet {
    panic!("If this messagge occurs, something went terribly wrong")
}

pub fn call_func(func: &MalType, args: &[MalType]) -> MalRet {
    match func {
        Fun(func, _) => func(args),
        MalFun {
            eval,
            params,
            ast,
            env,
            ..
        } => {
            let inner_env = env_binds(env.clone(), params, args)?;
            // It's fine to clone the environment here
            // since this is when the function is actually called
            match eval(ast, inner_env)? {
                List(list) => Ok(list.last().unwrap_or(&Nil).clone()),
                _ => scream(),
            }
        }
        _ => Err(MalErr::unrecoverable(
            format!("{:?} is not a function", prt(func)).as_str(),
        )),
    }
}

pub fn arithmetic_op(set: isize, f: fn(isize, isize) -> isize, args: &[MalType]) -> MalRet {
    if args.is_empty() {
        return Ok(Int(set));
    }

    let mut left = args[0].if_number()?;
    if args.len() > 1 {
        let right = &args[1..];
        for el in right {
            left = f(left, el.if_number()?);
        }
    }

    Ok(Int(left))
}

use MalType::{Bool, Nil};

pub fn comparison_op(f: fn(isize, isize) -> bool, args: &[MalType]) -> MalRet {
    let (left, rights) = car_cdr(args)?;
    let mut left = left.if_number()?;
    for right in rights {
        let right = right.if_number()?;
        if !f(left, right) {
            return Ok(Nil);
        }
        left = right;
    }
    Ok(Bool(true))
}

/// Extract the car and cdr from a list
pub fn car_cdr(list: &[MalType]) -> Result<(&MalType, &[MalType]), MalErr> {
    match list.len() {
        0 => Err(MalErr::unrecoverable("Expected at least one argument")),
        _ => Ok((
            &list[0],
            if list.len() > 1 {
                &list[1..]
            } else {
                &list[0..0]
            },
        )),
    }
}

use std::process::exit;

pub fn core_exit(list: &[MalType]) -> MalRet {
    match car_cdr(list)?.0 {
        Int(val) => exit(*val as i32),
        _ => exit(-1),
    }
}
