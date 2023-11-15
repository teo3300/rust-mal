use std::rc::Rc;

use crate::core::{call_func, car_cdr, scream};
use crate::env::Env;
use crate::env::{env_get, env_new, env_set};
use crate::printer::prt;
use crate::types::MalType::*;
use crate::types::{MalArgs, MalErr, MalMap, MalRet, MalType};

/// Resolve the first element of the list as the function name and call it
/// with the other elements as arguments
fn eval_func(list: &MalType) -> MalRet {
    match list {
        List(list) => {
            let (func, args) = car_cdr(list)?;
            call_func(func, args)
        }
        _ => scream(),
    }
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

/// def! special form:
///     Evaluate the second expression and assign it to the first symbol
fn def_bang_form(list: &[MalType], env: Env) -> MalRet {
    if list.len() != 2 {
        return Err(MalErr::unrecoverable("def! form: needs 2 arguments"));
    }
    match &list[0] {
        Sym(sym) => Ok(env_set(&env, sym.as_str(), &eval(&list[1], env.clone())?)),
        _ => Err(MalErr::unrecoverable(
            format!(
                "def! Assigning {:?} to {:?}, which is not a symbol",
                prt(&list[1]),
                prt(&list[0])
            )
            .as_str(),
        )),
    }
}

/// let* special form:
///     Create a temporary inner environment, assigning pair of elements in
///     the first list and returning the evaluation of the second expression
fn let_star_form(list: &[MalType], env: Env) -> MalRet {
    // Create the inner environment
    let inner_env = env_new(Some(env.clone()));
    // change the inner environment
    let (car, cdr) = car_cdr(list)?;
    match car {
        List(list) if list.len() % 2 == 0 => {
            // TODO: Find a way to avoid index looping that is ugly
            for i in (0..list.len()).step_by(2) {
                def_bang_form(&list[i..=i + 1], inner_env.clone())?;
            }
            if cdr.is_empty() {
                // TODO: check if it exists a better way to do this
                Ok(Nil)
            } else {
                eval(&cdr[0], inner_env)
            }
        }
        _ => Err(MalErr::unrecoverable(
            "First argument of let* must be a list of pair definitions",
        )),
    }
}

/// do special form:
///     Evaluate all the elements in a list using eval_ast and return the
///     result of the last evaluation
fn do_form(list: &[MalType], env: Env) -> MalRet {
    if list.is_empty() {
        return Err(MalErr::unrecoverable("do form: provide a list as argument"));
    }
    match eval_ast(&list[0], env)? {
        List(list) => Ok(list.last().unwrap_or(&Nil).clone()),
        _ => Err(MalErr::unrecoverable("do form: argument must be a list")),
    }
}

fn if_form(list: &[MalType], env: Env) -> MalRet {
    if !(2..=3).contains(&list.len()) {
        return Err(MalErr::unrecoverable(
            "if form: number of arguments is wrong",
        ));
    }
    let (cond, branches) = car_cdr(list)?;
    match eval(cond, env.clone())? {
        Nil | Bool(false) => match branches.len() {
            1 => Ok(Nil),
            _ => eval(&branches[1], env),
        },
        _ => eval(&branches[0], env),
    }
}

fn fn_star_form(list: &[MalType], env: Env) -> MalRet {
    let (binds, exprs) = car_cdr(list)?;
    Ok(MalFun {
        eval: eval_ast,
        params: Rc::new(binds.clone()),
        ast: Rc::new(List(exprs.to_vec())),
        env,
    })
}

use crate::printer::print_malfun;

pub fn help_form(list: &[MalType], env: Env) -> MalRet {
    let (sym, _) = car_cdr(list)?;
    match car_cdr(list)?.0 {
        Sym(sym_str) => {
            match eval(sym, env.clone())? {
                Fun(_, desc) => println!("{}\t[builtin]: {}", sym_str, desc),
                MalFun { params, ast, .. } => print_malfun(sym_str, params, ast),
                _ => println!("{}\t[symbol]: {}", sym_str, prt(&env_get(&env, sym_str)?)),
            }
            Ok(Bool(true))
        }
        _ => {
            println!("{} is not a symbol", prt(sym));
            Ok(Nil)
        }
    }
}

/// Intermediate function to discern special forms from defined symbols
fn apply(list: &MalArgs, env: Env) -> MalRet {
    let (car, cdr) = car_cdr(list)?;
    match car {
        Sym(sym) if sym == "def!" => def_bang_form(cdr, env), // Set for env
        Sym(sym) if sym == "let*" => let_star_form(cdr, env), // Clone the env
        Sym(sym) if sym == "do" => do_form(cdr, env),
        Sym(sym) if sym == "if" => if_form(cdr, env),
        Sym(sym) if sym == "fn*" => fn_star_form(cdr, env),
        Sym(sym) if sym == "help" => help_form(cdr, env),
        // Filter out special forms
        _ => eval_func(&eval_ast(&List(list.to_vec()), env)?),
    }
}

/// Switch ast evaluation depending on it being a list or not and return the
/// result of the evaluation, this function calls "eval_ast" to recursively
/// evaluate asts
pub fn eval(ast: &MalType, env: Env) -> MalRet {
    match &ast {
        List(list) if list.is_empty() => Ok(ast.clone()),
        List(list) if !list.is_empty() => apply(list, env),
        _ => eval_ast(ast, env),
    }
}

/// Separately evaluate all elements in a collection (list or vector)
fn eval_collection(list: &MalArgs, env: Env) -> Result<MalArgs, MalErr> {
    let mut ret = MalArgs::new();
    for el in list {
        ret.push(eval(el, env.clone())?);
    }
    Ok(ret)
}

/// Evaluate the values of a map
fn eval_map(map: &MalMap, env: Env) -> MalRet {
    let mut ret = MalMap::new();
    for (k, v) in map {
        ret.insert(k.to_string(), eval(v, env.clone())?);
    }
    Ok(Map(ret))
}

/// Eval the provided ast
fn eval_ast(ast: &MalType, env: Env) -> MalRet {
    match ast {
        Sym(sym) => env_get(&env, sym),
        List(list) => Ok(List(eval_collection(list, env)?)),
        Vector(vec) => Ok(Vector(eval_collection(vec, env)?)),
        Map(map) => eval_map(map, env),
        _ => Ok(ast.clone()),
    }
}
