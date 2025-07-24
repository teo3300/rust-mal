use crate::eval::eval;
use crate::types::MalErr;
use crate::types::{Frac, MalMap, MalRet, MalType};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct EnvType {
    data: RefCell<MalMap>,
    pub outer: Option<Env>,
}

impl EnvType {
    pub fn keys(&self) -> Vec<String> {
        let mut keys = self
            .data
            .borrow()
            .keys()
            .map(|k| k.to_string())
            .collect::<Vec<String>>();
        keys.sort_unstable();
        keys
    }
}

pub type Env = Rc<EnvType>;
// Following rust implementation, using shorthand to always pas Reference count

pub fn env_new(outer: Option<Env>) -> Env {
    Env::new(EnvType {
        data: RefCell::new(MalMap::new()),
        outer,
    })
}

pub fn env_set(env: &Env, sym: &str, val: &MalType) {
    env.data.borrow_mut().insert(sym.into(), val.clone());
}

pub fn env_get(env: &Env, sym: &str) -> MalRet {
    let mut iter_env = env;
    loop {
        if let Some(val) = iter_env.data.borrow().get(sym) {
            return Ok(val.clone());
        }
        if let Some(outer) = &iter_env.outer {
            iter_env = outer;
            continue;
        }
        return Err(MalErr::unrecoverable(
            format!("symbol {sym:?} not defined").as_str(),
        ));
    }
    // Recursive was prettier, but we hate recursion
}

pub fn env_binds(outer: Env, binds: &MalType, exprs: &[MalType]) -> Result<Env, MalErr> {
    let env = env_new(Some(outer));
    let binds = binds.if_list()?;
    let binl = binds.len();
    let expl = exprs.len();
    if binl < expl {
        return Err(MalErr::unrecoverable(
            format!("Expected {binl} args, got {expl}").as_str(),
        ));
    }
    for (bind, expr) in binds.iter().zip(exprs.iter()) {
        let bind = bind.if_symbol()?;
        env_set(&env, bind, expr);
    }
    // All arguments are optional, if an argument is not specified, set it to nil
    for bind in binds.iter().take(binl).skip(expl) {
        env_set(&env, bind.if_symbol()?, &M::Nil);
    }
    Ok(env)
}

macro_rules! scream {
    () => {
        panic!("If this messagge occurs, something went terribly wrong")
    };
}

use crate::printer::prt;
use crate::types::MalType as M;

pub enum CallFunc {
    Builtin(MalType),
    MalFun(MalType, Env),
}
pub type CallRet = Result<CallFunc, MalErr>;

pub fn call_func(func: &MalType, args: &[MalType]) -> CallRet {
    match func {
        M::Fun(func, _) => Ok(CallFunc::Builtin(func(args)?)),
        M::MalFun {
            // eval,
            params,
            ast,
            env,
            ..
        } => {
            let inner_env = env_binds(env.clone(), params, args)?;
            // It's fine to clone the environment here
            // since this is when the function is actually called
            match ast.as_ref() {
                M::List(list) => {
                    for x in &list[0..list.len() - 1] {
                        eval(x, inner_env.clone())?;
                    }
                    Ok(CallFunc::MalFun(
                        list.last().unwrap_or_default().clone(),
                        inner_env,
                    ))
                }
                _ => scream!(),
            }
        }
        M::Map(m) => {
            if args.is_empty() {
                return Err(MalErr::unrecoverable("No key provided to Map construct"));
            }
            match &args[0] {
                M::Str(s) | M::Key(s) => {
                    Ok(CallFunc::Builtin(m.get(s).unwrap_or_default().clone()))
                }
                _ => Err(MalErr::unrecoverable("Map argument must be string or key")),
            }
        }
        M::Vector(v) => {
            if args.is_empty() {
                return Err(MalErr::unrecoverable("No key provided to Vector construct"));
            }
            match &args[0] {
                M::Num(i) => {
                    if { 0..v.len() as isize }.contains(&i.int()) {
                        Ok(CallFunc::Builtin(v[i.int() as usize].clone()))
                    } else {
                        Ok(CallFunc::Builtin(M::Nil))
                    }
                }
                _ => Err(MalErr::unrecoverable("Map argument must be string or key")),
            }
        }
        _ => Err(MalErr::unrecoverable(
            format!("{:?} is not a function", prt(func)).as_str(),
        )),
    }
}

pub fn any_zero(list: &[MalType]) -> Result<&[MalType], MalErr> {
    match list.len() {
        1 => {
            if list[0].if_number()?.get_num() == 0 {
                Err(MalErr::unrecoverable("Attempting division by 0"))
            } else {
                Ok(list)
            }
        }
        _ => {
            if list[1..list.len()]
                .iter()
                .any(|x| matches!(x, M::Num(v) if v.exact_zero()))
            {
                Err(MalErr::unrecoverable("Attempting division by 0"))
            } else {
                Ok(list)
            }
        }
    }
}

pub fn arithmetic_op(set: isize, f: fn(Frac, Frac) -> Frac, args: &[MalType]) -> MalRet {
    Ok(M::Num(match args.len() {
        0 => Frac::num(set),
        1 => f(Frac::num(set), args[0].if_number()?),
        _ => {
            // TODO: Maybe an accumulator
            let mut left = args[0].if_number()?;
            for el in &args[1..] {
                left = f(left, el.if_number()?);

                const SIM_TRIG: usize = usize::isqrt(isize::MAX as usize);

                // TODO: consider if simplifying at every operation or every N
                //          or just at the end, no idea on how it scale
                if std::cmp::max(left.get_num().unsigned_abs(), left.get_den()) > SIM_TRIG {
                    left = left.simplify();
                }
            }
            // Always simplify at the end
            left.simplify()
        }
    }))
}

use MalType::{Nil, T};
pub fn comparison_op(f: fn(Frac, Frac) -> bool, args: &[MalType]) -> MalRet {
    if args.is_empty() {
        return Ok(Nil);
    }
    let (left, rights) = car_cdr(args)?;
    let mut left = left.if_number()?;
    for right in rights {
        let right = right.if_number()?;
        if !f(left, right) {
            return Ok(Nil);
        }
        left = right;
    }
    Ok(T)
}

pub fn car(list: &[MalType]) -> Result<&MalType, MalErr> {
    match list.len() {
        0 => Err(MalErr::unrecoverable("Expected at least one argument")),
        _ => Ok(&list[0]),
    }
}

pub fn cdr(list: &[MalType]) -> &[MalType] {
    if list.len() > 1 {
        &list[1..]
    } else {
        &list[0..0]
    }
}

pub fn mal_cdr(arg: &MalType) -> MalRet {
    let list = arg.if_list()?;
    Ok(MalType::List(cdr(list).into()))
}

pub fn mal_car(arg: &MalType) -> MalRet {
    let list = arg.if_list()?;
    if list.is_empty() {
        Ok(Nil)
    } else {
        Ok(list[0].clone())
    }
}

/// Extract the car and cdr from a list
pub fn car_cdr(list: &[MalType]) -> Result<(&MalType, &[MalType]), MalErr> {
    Ok((car(list)?, cdr(list)))
}

// TODO: fix these chonky functions

pub fn mal_cons(args: &[MalType]) -> MalRet {
    match args.len() {
        2 => {
            let mut car = vec![args[0].clone()];
            let cdr = args[1].if_list()?;
            car.extend_from_slice(cdr);
            Ok(M::List(car.into()))
        }
        _ => Err(MalErr::unrecoverable("cons: requires 2 arguments")),
    }
}

fn first(list: &[MalType]) -> &[MalType] {
    if list.len() > 1 {
        &list[..list.len() - 1]
    } else {
        &list[0..0]
    }
}

// FIXME: Treat as result for now, change later
fn last(list: &[MalType]) -> Result<&MalType, MalErr> {
    match list.len() {
        0 => Ok(&MalType::Nil),
        _ => Ok(&list[list.len() - 1]),
    }
}

pub fn first_last(list: &[MalType]) -> (&[MalType], Result<&MalType, MalErr>) {
    (first(list), last(list))
}

use std::process::exit;

pub fn mal_exit(list: &[MalType]) -> MalRet {
    match car(list)? {
        MalType::Num(val) => exit(val.int() as i32),
        _ => exit(-1),
    }
}

// TODO: find another way to process strings
pub fn mal_boom(args: &[MalType]) -> MalRet {
    let string = car(args)?.if_string()?;
    Ok(M::List(string.chars().map(M::Ch).collect()))
}
