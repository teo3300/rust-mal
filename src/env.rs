use crate::types::MalErr;
use crate::types::{MalMap, MalRet, MalType};
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
            .iter()
            .map(|(k, _)| k.clone())
            .collect::<Vec<String>>();
        keys.sort_unstable();
        keys
    }
}

pub type Env = Rc<EnvType>;
// Following rust implementation, using shorthand to always pas Reference count

pub fn env_new(outer: Option<Env>) -> Env {
    Rc::new(EnvType {
        data: RefCell::new(MalMap::new()),
        outer,
    })
}

pub fn env_set(env: &Env, sym: &str, val: &MalType) -> MalType {
    env.data.borrow_mut().insert(sym.to_string(), val.clone());
    val.clone()
}

pub fn env_get(env: &Env, sym: &str) -> MalRet {
    match env.data.borrow().get(sym) {
        Some(val) => Ok(val.clone()),
        None => match env.outer.clone() {
            Some(outer) => env_get(&outer, sym),
            None => Err(MalErr::unrecoverable(
                format!("symbol {:?} not defined", sym).as_str(),
            )),
        },
    }
}

pub fn env_binds(outer: Env, binds: &MalType, exprs: &[MalType]) -> Result<Env, MalErr> {
    let env = env_new(Some(outer));
    let binds = binds.if_vec()?;
    if binds.len() != exprs.len() {
        return Err(MalErr::unrecoverable(
            format!("Expected {} args, got {}", binds.len(), exprs.len()).as_str(),
        ));
    } // TODO: May be possible to leave this be and not set additional elements at all
    for (bind, expr) in binds.iter().zip(exprs.iter()) {
        let bind = bind.if_symbol()?;
        env_set(&env, bind, expr);
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
                M::List(list) => Ok(CallFunc::MalFun(
                    list.last().unwrap_or(&Nil).clone(),
                    inner_env,
                )),
                _ => scream!(),
            }
        }
        _ => Err(MalErr::unrecoverable(
            format!("{:?} is not a function", prt(func)).as_str(),
        )),
    }
}

pub fn any_zero(list: &[MalType]) -> MalRet {
    if list.iter().any(|x| matches!(x, MalType::Int(0))) {
        return Err(MalErr::unrecoverable("Attempting division by 0"));
    }
    Ok(M::Nil)
}

pub fn arithmetic_op(set: isize, f: fn(isize, isize) -> isize, args: &[MalType]) -> MalRet {
    Ok(M::Int(match args.len() {
        0 => set,
        1 => f(set, args[0].if_number()?),
        _ => {
            // TODO: Maybe an accumulator
            let mut left = args[0].if_number()?;
            for el in &args[1..] {
                left = f(left, el.if_number()?);
            }
            left
        }
    }))
}

use MalType::{Bool, Nil};

pub fn comparison_op(f: fn(isize, isize) -> bool, args: &[MalType]) -> MalRet {
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
    Ok(Bool(true))
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

/// Extract the car and cdr from a list
pub fn car_cdr(list: &[MalType]) -> Result<(&MalType, &[MalType]), MalErr> {
    Ok((car(list)?, cdr(list)))
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
        0 => Err(MalErr::unrecoverable("Mi sono cacato le mutande")),
        _ => Ok(&list[list.len() - 1]),
    }
}

pub fn first_last(list: &[MalType]) -> (&[MalType], Result<&MalType, MalErr>) {
    (first(list), last(list))
}

use std::process::exit;

pub fn mal_exit(list: &[MalType]) -> MalRet {
    match car_cdr(list)?.0 {
        MalType::Int(val) => exit(*val as i32),
        _ => exit(-1),
    }
}
