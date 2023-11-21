use std::rc::Rc;

use crate::env::Env;
use crate::env::{call_func, car_cdr};
use crate::env::{env_get, env_new, env_set};
use crate::printer::prt;
use crate::types::MalType as M;
use crate::types::{MalArgs, MalErr, MalMap, MalRet, MalType};

/// Resolve the first element of the list as the function name and call it
/// with the other elements as arguments
fn eval_func(list: &MalType) -> MalRet {
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

/// def! special form:
///     Evaluate the second expression and assign it to the first symbol
fn def_bang_form(list: &[MalType], env: Env) -> MalRet {
    if list.len() != 2 {
        return Err(MalErr::unrecoverable("def! form: needs 2 arguments"));
    }
    let (car, _) = car_cdr(list)?;
    let sym = car.if_symbol()?;
    Ok(env_set(&env, sym, &eval(&list[1], env.clone())?))
}

/// let* special form:
///     Create a temporary inner environment, assigning pair of elements in
///     the first list and returning the evaluation of the second expression
fn let_star_form(list: &[MalType], env: Env) -> MalRet {
    // Create the inner environment
    let inner_env = env_new(Some(env.clone()));
    // change the inner environment
    let (car, cdr) = car_cdr(list)?;
    let list = car.if_list()?;
    if list.len() % 2 != 0 {
        return Err(MalErr::unrecoverable(
            "let* form, number of arguments must be even",
        ));
    }
    // TO-DO: Find a way to avoid index looping that is ugly
    for i in (0..list.len()).step_by(2) {
        def_bang_form(&list[i..=i + 1], inner_env.clone())?;
    }
    let mut last = M::Nil;
    for expr in cdr {
        last = eval(expr, inner_env.clone())?;
    }
    Ok(last)
}

/// do special form:
///     Evaluate all the elements in a list using eval_ast and return the
///     result of the last evaluation
fn do_form(list: &[MalType], env: Env) -> MalRet {
    if list.is_empty() {
        return Err(MalErr::unrecoverable("do form: provide a list as argument"));
    }
    // TODO: this may be different

    let mut ret = M::Nil;
    for ast in list {
        ret = eval(ast, env.clone())?;
    }
    Ok(ret)
}

fn if_form(list: &[MalType], env: Env) -> MalRet {
    if !(2..=3).contains(&list.len()) {
        return Err(MalErr::unrecoverable(
            "if form: number of arguments is wrong",
        ));
    }
    let (cond, branches) = car_cdr(list)?;
    match eval(cond, env.clone())? {
        M::Nil | M::Bool(false) => match branches.len() {
            1 => Ok(M::Nil),
            _ => eval(&branches[1], env),
        },
        _ => eval(&branches[0], env),
    }
}

fn fn_star_form(list: &[MalType], env: Env) -> MalRet {
    let (binds, exprs) = car_cdr(list)?;
    binds.if_list()?;
    Ok(M::MalFun {
        eval: eval_ast,
        params: Rc::new(binds.clone()),
        ast: Rc::new(M::List(exprs.to_vec())),
        env,
    })
}

use crate::printer::print_malfun;

pub fn help_form(list: &[MalType], env: Env) -> MalRet {
    let (sym, _) = car_cdr(list)?;
    let sym_str = sym.if_symbol()?;
    match eval(sym, env.clone())? {
        M::Fun(_, desc) => println!("{}\t[builtin]: {}", sym_str, desc),
        M::MalFun { params, ast, .. } => print_malfun(sym_str, params, ast),
        _ => println!("{}\t[symbol]: {}", sym_str, prt(&env_get(&env, sym_str)?)),
    }
    Ok(M::Bool(true))
}

/// Intermediate function to discern special forms from defined symbols
fn apply(list: &MalArgs, env: Env) -> MalRet {
    let (car, cdr) = car_cdr(list)?;
    match car {
        M::Sym(sym) if sym == "def!" => def_bang_form(cdr, env), // Set for env
        M::Sym(sym) if sym == "let*" => let_star_form(cdr, env), // Clone the env
        M::Sym(sym) if sym == "do" => do_form(cdr, env),
        M::Sym(sym) if sym == "if" => if_form(cdr, env),
        M::Sym(sym) if sym == "fn*" => fn_star_form(cdr, env),
        M::Sym(sym) if sym == "help" => help_form(cdr, env),
        // Filter out special forms
        _ => eval_func(&eval_ast(&M::List(list.to_vec()), env)?),
    }
}

/// Switch ast evaluation depending on it being a list or not and return the
/// result of the evaluation, this function calls "eval_ast" to recursively
/// evaluate asts
pub fn eval(ast: &MalType, env: Env) -> MalRet {
    match &ast {
        M::List(list) if list.is_empty() => Ok(ast.clone()),
        M::List(list) if !list.is_empty() => apply(list, env),
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
