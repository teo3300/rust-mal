use crate::env::{self, call_func, car, car_cdr, CallFunc, CallRet};
use crate::env::{env_get, env_new, env_set};
use crate::env::{first_last, Env};
use crate::printer::prt;
use crate::types::MalType as M;
use crate::types::{MalArgs, MalErr, MalMap, MalRet, MalType};
use std::borrow::Borrow;
use std::rc::Rc;

macro_rules! forms {
    ($($name:ident : $value:expr),*) => {
        $(
            const $name: &'static str = $value;
        )*
    };
}
forms!(NAME_DEF     : "def!",
       NAME_LET     : "let*",
       NAME_DO      : "do",
       NAME_IF      : "if",
       NAME_FN      : "fn*",
       NAME_FN_ALT  : "λ", 
       NAME_HELP    : "help",
       NAME_HELP_ALT: "h",
       NAME_FIND    : "find",
       NAME_QUOTE   : "quote",
       NAME_OK      : "ok?",
       NAME_EVAL    : "eval");

/// Resolve the first element of the list as the function name and call it
/// with the other elements as arguments
fn eval_func(list: &MalType) -> CallRet {
    let list = list.if_list()?;
    let (func, args) = car_cdr(list)?;
    call_func(func, args)
}

// When evaluating an expression it's possible
// (actually the only option I'm aware until now)
// to clone the environment "env.clone()", performances aside
// [env.clone()]
// NOTE: Ok, so cloning an Env now is not bad, since it's an Rc
// (Just pointing this out because I know I will try to change this later)
//
// It's not possible, however, to clone the outer when defining
// a new environment that will be used later (such as when using fn*)

macro_rules! inner_do {
    ($list:expr, $env:expr) => {{
        let (first, last) = first_last($list);
        for ast in first {
            eval(ast, $env.clone())?;
        }
        last.cloned()
    }};
}

/// def! special form:
///     Evaluate the second expression and assign it to the first symbol
fn def_bang_form(list: &[MalType], env: Env) -> MalRet {
    if list.len() != 2 {
        return Err(MalErr::unrecoverable("def! form: needs 2 arguments"));
    }
    let (car, _) = car_cdr(list)?;
    let sym = car.if_symbol()?;
    let val = &eval(&list[1], env.clone())?;
    env_set(&env, sym, val);
    Ok(val.clone())
}

/// let* special form:
///     Create a temporary inner environment, assigning pair of elements in
///     the first list and returning the evaluation of the second expression
fn let_star_form(list: &[MalType], env: Env) -> Result<(MalType, Env), MalErr> {
    // Create the inner environment
    let inner_env = env_new(Some(env.clone()));
    // change the inner environment
    let (car, cdr) = car_cdr(list)?;
    let list = car.if_vec()?;
    if list.len() % 2 != 0 {
        return Err(MalErr::unrecoverable(
            "let* form, number of arguments must be even",
        ));
    }
    // TO-DO: Find a way to avoid index looping that is ugly
    for i in (0..list.len()).step_by(2) {
        def_bang_form(&list[i..=i + 1], inner_env.clone())?;
    }

    Ok((inner_do!(cdr, inner_env)?, inner_env))
}

/// do special form:
///     Evaluate all the elements in a list using eval_ast and return the
///     result of the last evaluation
fn do_form(list: &[MalType], env: Env) -> MalRet {
    inner_do!(list, env)
}

fn if_form(list: &[MalType], env: Env) -> MalRet {
    if !(2..=3).contains(&list.len()) {
        return Err(MalErr::unrecoverable(
            "if form: number of arguments is wrong",
        ));
    }
    let (cond, branches) = car_cdr(list)?;
    Ok(match eval(cond, env.clone())? {
        M::Nil | M::Bool(false) => match branches.len() {
            1 => M::Nil,
            _ => branches[1].clone(),
        },
        _ => branches[0].clone(),
    })
}

fn fn_star_form(list: &[MalType], env: Env) -> MalRet {
    let (binds, exprs) = car_cdr(list)?;
    binds.if_vec()?;
    Ok(M::MalFun {
        // eval: eval_ast,
        params: Rc::new(binds.clone()),
        ast: Rc::new(M::List(exprs.into())),
        env,
    })
}

use crate::printer::print_malfun;

pub fn help_form(list: &[MalType], env: Env) -> MalRet {
    let (sym, _) = car_cdr(list)?;
    let sym_str = sym.if_symbol()?;
    match eval(sym, env.clone())? {
        M::Fun(_, desc) => println!("{}\t[builtin]: {}\n", sym_str, desc),
        M::MalFun { params, ast, .. } => print_malfun(sym_str, params, ast),
        _ => eprintln!("{}\t[symbol]: {}\n", sym_str, prt(&env_get(&env, sym_str)?)),
    }
    Ok(M::Nil)
}

pub fn find_form(list: &[MalType], env: Env) -> MalRet {
    let mut filtered = env.keys();
    for mat in list {
        let mat = mat.if_symbol()?;
        filtered.retain(|x| x.contains(mat));
    }
    eprintln!("\t[matches]:\n{}\n", filtered.join(" "));
    Ok(M::Nil)
}

pub fn outermost(env: &Env) -> Env {
    let mut env = env;
    while let Some(ref e) = env.outer {
        env = e;
    }
    env.clone()
}

macro_rules! apply {
    ($ast:expr, $env:expr) => {{
        let apply_list = &eval_ast(&$ast, $env.clone())?;
        let eval_ret = eval_func(apply_list)?;

        match eval_ret {
            CallFunc::Builtin(ret) => return Ok(ret),
            CallFunc::MalFun(fun_ast, fun_env) => {
                $ast = fun_ast;
                $env = fun_env;
            }
        }
    }};
}

/// Intermediate function to discern special forms from defined symbols
pub fn eval(ast: &MalType, env: Env) -> MalRet {
    let mut ast = ast.clone();
    let mut env = env;
    loop {
        match &ast {
            M::List(list) if list.is_empty() => return Ok(ast.clone()),
            M::List(list) => {
                let (symbol, args) = car_cdr(list)?;
                if let M::Sym(sym) = symbol {
                    match sym.borrow() {
                        // I don't like to borrow tho
                        NAME_DEF => return def_bang_form(args, env.clone()), // Set for env
                        NAME_LET => {(ast, env) = let_star_form(args, env.clone())?; continue;},
                        NAME_DO  => {ast = do_form(args, env.clone())?; continue;},
                        NAME_IF  => {ast = if_form(args, env.clone())?; continue;},
                        NAME_FN | NAME_FN_ALT /* :) */ => {
                            return fn_star_form(args, env.clone())
                        }
                        NAME_HELP | NAME_HELP_ALT => return help_form(args, env.clone()),
                        NAME_FIND => return find_form(args, env.clone()),
                        // Oh God, what have I done
                        NAME_QUOTE => return Ok(car(args)?.clone()),
                        NAME_OK => {
                            return match eval(car(args)?, env.clone()) {
                                Err(_) => Ok(M::Nil),
                                _ => Ok(M::Bool(true)),
                            }
                        }
                        // Special form, sad
                        // Bruh, is basically double eval
                        NAME_EVAL => {
                            ast = eval(env::car(args)?, env.clone())?;
                            // Climb to the outermost environment (The repl env)
                            env = outermost(&env);
                            continue;
                        }
                        _ => {}
                    }
                }
                // "apply"/invoke
                apply!(ast, env)
            }
            _ => return eval_ast(&ast, env),
        }
    }
}

/// Separately evaluate all elements in a collection (list or vector)
fn eval_collection(list: &MalArgs, env: Env) -> Result<MalArgs, MalErr> {
    let mut ret = Vec::new();
    for el in list.as_ref() {
        ret.push(eval(el, env.clone())?);
    }
    Ok(ret.into())
}

/// Evaluate the values of a map
fn eval_map(map: &MalMap, env: Env) -> MalRet {
    let mut ret = MalMap::new();
    for (k, v) in map {
        ret.insert(k.clone(), eval(v, env.clone())?);
    }
    Ok(M::Map(ret))
}

/// Eval the provided ast
fn eval_ast(ast: &MalType, env: Env) -> MalRet {
    match ast {
        M::Sym(sym) => env_get(&env, sym),
        M::List(list) => Ok(M::List(eval_collection(list, env)?)),
        M::Vector(vec) => Ok(M::Vector(eval_collection(vec, env)?)),
        M::Map(map) => eval_map(map, env),
        _ => Ok(ast.clone()),
    }
}

// all tests moved to mal
