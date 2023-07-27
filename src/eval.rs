use crate::env::env_binds;
use crate::env::Env;
use crate::types::car_cdr;
use crate::types::MalType::*;
use crate::types::{MalArgs, MalMap, MalRet, MalType};

fn call_func(func: &MalType, args: &[MalType]) -> MalRet {
    match func {
        Fun(func) => func(args),
        MalFun {
            eval,
            params,
            ast,
            env,
        } => {
            let mut inner_env = env_binds(env, &**params, args)?;
            let mut ret = Ok(Nil);
            for el in ast.iter() {
                ret = eval(el, &mut inner_env);
            }
            ret
        }
        _ => Err(format!("{:?} is not a function", func)),
    }
}

/// Resolve the first element of the list as the function name and call it
/// with the other elements as arguments
fn eval_func(list: &MalType) -> MalRet {
    match list {
        List(list) => {
            let (func, args) = car_cdr(list);
            call_func(func, args)
        }
        _ => todo!("This should never happen, if you see this message I probably broke the code"),
    }
}

/// def! special form:
///     Evaluate the second expression and assign it to the first symbol
fn def_bang_form(list: &[MalType], env: &mut Env) -> MalRet {
    match list.len() {
        2 => match &list[0] {
            Sym(sym) => {
                let cdr = eval(&list[1], env)?;
                env.set(sym.as_str(), &cdr);
                Ok(cdr)
            }
            _ => Err(format!(
                "def! Assigning {:?} to {:?}, which is not a symbol",
                &list[1], &list[0]
            )),
        },
        _ => Err("def! form: needs 2 arguments".to_string()),
    }
}

/// let* special form:
///     Create a temporary inner environment, assigning pair of elements in
///     the first list and returning the evaluation of the second expression
fn let_star_form(list: &[MalType], env: &Env) -> MalRet {
    // Create the inner environment
    let mut inner_env = Env::new(Some(Box::new(env.clone())));
    // change the inner environment
    let (car, cdr) = car_cdr(list);
    match car {
        List(list) if list.len() % 2 == 0 => {
            // TODO: Find a way to avoid index looping that is ugly
            for i in (0..list.len()).step_by(2) {
                def_bang_form(&list[i..i + 2], &mut inner_env)?;
            }
            if cdr.is_empty() {
                // TODO: check if it exists a better way to do this
                Ok(Nil)
            } else {
                eval(&cdr[0], &mut inner_env)
            }
        }
        _ => Err("First argument of let* must be a list of pair definitions".to_string()),
    }
}

/// do special form:
///     Evaluate all the elements in a list using eval_ast and return the
///     result of the last evaluation
fn do_form(list: &[MalType], env: &mut Env) -> MalRet {
    if list.is_empty() {
        return Err("do form: provide a list as argument".to_string());
    }
    match eval_ast(&list[0], env)? {
        List(list) => Ok(list.last().unwrap_or(&Nil).clone()),
        _ => Err("do form: argument must be a list".to_string()),
    }
}

fn if_form(list: &[MalType], env: &mut Env) -> MalRet {
    if !(2..=3).contains(&list.len()) {
        return Err("if form: number of arguments".to_string());
    }
    let (cond, branches) = car_cdr(list);
    match eval(cond, env)? {
        Nil | Bool(false) => match branches.len() {
            1 => Ok(Nil),
            _ => eval(&branches[1], env),
        },
        _ => eval(&branches[0], env),
    }
}

fn fn_star_form(list: &[MalType], env: &Env) -> MalRet {
    if list.is_empty() {
        return Err("fn* form: specify lambda arguments".to_string());
    }
    let (binds, exprs) = car_cdr(list);
    Ok(MalFun {
        eval: eval,
        params: Box::new(binds.clone()),
        ast: Box::new(exprs.to_vec()),
        env: env.clone(),
    })
}

/// Intermediate function to discern special forms from defined symbols
fn apply(list: &MalArgs, env: &mut Env) -> MalRet {
    let (car, cdr) = car_cdr(list);
    match car {
        Sym(sym) if sym == "def!" => def_bang_form(cdr, env),
        Sym(sym) if sym == "let*" => let_star_form(cdr, env),
        Sym(sym) if sym == "do" => do_form(cdr, env),
        Sym(sym) if sym == "if" => if_form(cdr, env),
        Sym(sym) if sym == "fn*" => fn_star_form(cdr, env),
        // Filter out special forms
        _ => eval_func(&eval_ast(&List(list.to_vec()), env)?),
    }
}

/// Switch ast evaluation depending on it being a list or not and return the
/// result of the evaluation, this function calls "eval_ast" to recursively
/// evaluate asts
pub fn eval(ast: &MalType, env: &mut Env) -> MalRet {
    match &ast {
        List(list) if list.is_empty() => Ok(ast.clone()),
        List(list) if !list.is_empty() => apply(list, env),
        _ => eval_ast(ast, env),
    }
}

/// Separetely evaluate all elements in a collection (list or vector)
fn eval_collection(list: &MalArgs, env: &mut Env) -> Result<MalArgs, String> {
    let mut ret = MalArgs::new();
    for el in list {
        ret.push(eval(el, env)?);
    }
    Ok(ret)
}

/// Evaluate the values of a map
fn eval_map(map: &MalMap, env: &mut Env) -> MalRet {
    let mut ret = MalMap::new();
    for (k, v) in map {
        ret.insert(k.to_string(), eval(v, env)?);
    }
    Ok(Map(ret))
}

/// Eval the provided ast
fn eval_ast(ast: &MalType, env: &mut Env) -> MalRet {
    match ast {
        Sym(sym) => env.get(sym),
        List(list) => Ok(List(eval_collection(list, env)?)),
        Vector(vec) => Ok(Vector(eval_collection(vec, env)?)),
        Map(map) => eval_map(map, env),
        _ => Ok(ast.clone()),
    }
}
