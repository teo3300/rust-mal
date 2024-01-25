use crate::env::{self, call_func, car, car_cdr, CallFunc, CallRet};
use crate::env::{env_get, env_new, env_set};
use crate::env::{first_last, Env};
use crate::printer::prt;
use crate::types::MalType as M;
use crate::types::{MalArgs, MalErr, MalMap, MalRet, MalType};
use std::rc::Rc;

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
        ast: Rc::new(M::List(MalArgs::new(exprs.to_vec()))),
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

/// Intermediate function to discern special forms from defined symbols
pub fn eval(ast: &MalType, env: Env) -> MalRet {
    let mut ast = ast.clone();
    let mut env = env;
    loop {
        match &ast {
            M::List(list) if list.is_empty() => return Ok(ast.clone()),
            M::List(list) => {
                let (symbol, args) = car_cdr(list)?;
                match symbol {
                    M::Sym(sym) if sym == "def!" => return def_bang_form(args, env.clone()), // Set for env
                    M::Sym(sym) if sym == "let*" => (ast, env) = let_star_form(args, env.clone())?,
                    M::Sym(sym) if sym == "do" => ast = do_form(args, env.clone())?,
                    M::Sym(sym) if sym == "if" => ast = if_form(args, env.clone())?,
                    M::Sym(sym) if sym == "fn*" || sym == "λ" /* :) */ => {
                        return fn_star_form(args, env.clone())
                    }
                    M::Sym(sym) if sym == "help" => return help_form(args, env.clone()),
                    M::Sym(sym) if sym == "find" => return find_form(args, env.clone()),
                    // Oh God, what have I done
                    M::Sym(sym) if sym == "quote" => return Ok(car(args)?.clone()),
                    M::Sym(sym) if sym == "ok?" => {
                        return match eval(car(args)?, env.clone()) {
                            Err(_) => Ok(M::Nil),
                            _ => Ok(M::Bool(true)),
                        }
                    }
                    // Special form, sad
                    // Bruh, is basically double eval
                    M::Sym(sym) if sym == "eval" => {
                        ast = eval(env::car(args)?, env.clone())?;
                        // Climb to the outermost environment (The repl env)
                        env = outermost(&env);
                    }
                    // Filter out special forms
                    // "apply"/invoke
                    _ => {
                        let apply_list = &eval_ast(&ast, env.clone())?;
                        let eval_ret = eval_func(apply_list)?;

                        match eval_ret {
                            CallFunc::Builtin(ret) => return Ok(ret),
                            CallFunc::MalFun(fun_ast, fun_env) => {
                                ast = fun_ast;
                                env = fun_env;
                            }
                        }
                    }
                };
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
    Ok(MalArgs::new(ret))
}

/// Evaluate the values of a map
fn eval_map(map: &MalMap, env: Env) -> MalRet {
    let mut ret = MalMap::new();
    for (k, v) in map {
        ret.insert(k.to_string(), eval(v, env.clone())?);
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
