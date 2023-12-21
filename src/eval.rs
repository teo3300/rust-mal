use crate::env::{call_func, car_cdr, CallFunc, CallRet};
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
    Ok(env_set(&env, sym, &eval(&list[1], env.clone())?))
}

/// let* special form:
///     Create a temporary inner environment, assigning pair of elements in
///     the first list and returning the evaluation of the second expression
fn let_star_form(list: &[MalType], env: Env) -> Result<(MalType, Env), MalErr> {
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
    binds.if_list()?;
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
        M::Fun(_, desc) => println!("{}\t[builtin]: {}", sym_str, desc),
        M::MalFun { params, ast, .. } => print_malfun(sym_str, params, ast),
        _ => println!("{}\t[symbol]: {}", sym_str, prt(&env_get(&env, sym_str)?)),
    }
    Ok(M::Bool(true))
}

/// Intermediate function to discern special forms from defined symbols
pub fn eval(ast: &MalType, env: Env) -> MalRet {
    let mut ast = ast.clone();
    let mut env = env;
    loop {
        match &ast {
            M::List(list) if list.is_empty() => return Ok(ast.clone()),
            M::List(list) if !list.is_empty() => {
                let (car, cdr) = car_cdr(list)?;
                match car {
                    M::Sym(sym) if sym == "def!" => return def_bang_form(cdr, env.clone()), // Set for env
                    M::Sym(sym) if sym == "let*" => (ast, env) = let_star_form(cdr, env.clone())?,
                    M::Sym(sym) if sym == "do" => ast = do_form(cdr, env.clone())?,
                    M::Sym(sym) if sym == "if" => ast = if_form(cdr, env.clone())?,
                    M::Sym(sym) if sym == "fn*" => return fn_star_form(cdr, env.clone()),
                    M::Sym(sym) if sym == "help" => return help_form(cdr, env.clone()),
                    // Filter out special forms
                    // "apply"/invoke
                    _ => {
                        let apply_list =
                            &eval_ast(&M::List(MalArgs::new(list.to_vec())), env.clone())?;
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

/// Switch ast evaluation depending on it being a list or not and return the
/// result of the evaluation, this function calls "eval_ast" to recursively
/// evaluate asts
/*pub fn eval(ast: &MalType, env: Env) -> MalRet {
    match &ast {
        M::List(list) if list.is_empty() => Ok(ast.clone()),
        M::List(list) if !list.is_empty() => apply(list, env),
        _ => eval_ast(ast, env),
    }
}*/

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

////////////////////////////////////////////////////////////////////////////////
// Tests                                                                      //
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {

    use crate::env::Env;

    macro_rules! load2 {
        ($input:expr) => {{
            use crate::reader::{read_str, Reader};

            let r = Reader::new();
            r.push($input);
            &match read_str(&r) {
                Ok(v) => match v {
                    MalType::List(v) => v,
                    _ => panic!("Not a list"),
                },
                _ => panic!("Bad command"),
            }
        }};
    }

    macro_rules! load {
        ($input:expr) => {{
            use crate::env::cdr;
            cdr(load2!($input))
        }};
    }

    /*macro_rules! load_f {
        ($input:expr, $env:expr) => {{
            use crate::reader::{read_str, Reader};
            use std::rc::Rc;

            let r = Reader::new();
            r.push($input);
            let args = match read_str(&r) {
                Ok(v) => match v {
                    MalType::List(v) => v,
                    _ => panic!("Bad command"),
                },
                _ => panic!("Bad command"),
            };
            &MalType::List(Rc::new(if args.is_empty() {
                Vec::new()
            } else {
                let f_str = match &args[0] {
                    MalType::Sym(s) => s.as_str(),
                    _ => panic!("Can't solve function"),
                };
                let f = match env_get(&$env.clone(), f_str) {
                    Ok(v) => v,
                    _ => panic!("No such function in env"),
                };
                [&[f], &args[1..]].concat()
            }))
        }};
    }*/

    fn _env_empty() -> Env {
        use crate::env::env_new;
        env_new(None)
    }

    mod forms {
        use crate::env::env_get;
        use crate::eval::tests::_env_empty;
        use crate::eval::{def_bang_form, fn_star_form, if_form, let_star_form};
        use crate::types::MalType;

        #[test]
        fn def_bang() {
            let env = _env_empty();

            assert!(matches!(   //x empty
                def_bang_form(
                    load!("(def!) ; empty"), 
                    env.clone()),
                Err(e)
                if !e.is_recoverable()));
            assert!(matches!(   //x 1 arg
                def_bang_form(
                    load!("(def! a) ; 1 arg"), 
                    env.clone()),
                Err(e)
                if !e.is_recoverable()));
            assert!(matches!(   //x 3 args
                def_bang_form(
                    load!("(def! a 1 2) ; 3 args"), 
                    env.clone()),
                Err(e)
                if !e.is_recoverable()));

            assert!(matches!(
                //v 2 args
                def_bang_form(load!("(def! a 1) ; correct a = 1"), env.clone()),
                Ok(MalType::Int(1))
            ));
            assert!(matches!(env_get(&env, "a"), Ok(MalType::Int(1))));
        }

        #[test]
        fn let_star() {
            let env = _env_empty();
            assert!(
                matches!(let_star_form(load!("(let*)"), env.clone()), Err(e) if !e.is_recoverable())
            );
            assert!(
                matches!(let_star_form(load!("(let* 1)"), env.clone()), Err(e) if !e.is_recoverable())
            );
            assert!(
                matches!(let_star_form(load!("(let* (a))"), env.clone()), Err(e) if !e.is_recoverable())
            ); /*
               assert!(matches!(
                   let_star_form(load!("(let* ())"), env.clone()),
                   Ok(MalType::Nil)
               ));
               assert!(matches!(
                   let_star_form(load!("(let* (a 1))"), env.clone()),
                   Ok(MalType::Nil)
               ));
               assert!(matches!(env_get(&env.clone(), "a"), Err(e) if !e.is_recoverable()));
               assert!(matches!(
                   let_star_form(load!("(let* (a 1 b 2) a b)"), env.clone()),
                   Ok(MalType::Int(2))
               ));
               assert!(matches!(env_get(&env.clone(), "a"), Err(e) if !e.is_recoverable()));
               assert!(matches!(env_get(&env.clone(), "b"), Err(e) if !e.is_recoverable()));
               assert!(matches!(
                   let_star_form(load!("(let* (a 1 b 2) (def! c 1) a b)"), env.clone()),
                   Ok(MalType::Int(2))
               ));*/
            assert!(matches!(env_get(&env.clone(), "c"), Err(e) if !e.is_recoverable()));
        }

        /*#[test]
        fn _do_form() {
            let env = _env_empty();
            assert!(matches!(
                do_form(load!("(do)"), env.clone()),
                Ok(MalType::Nil)
            ));
            assert!(matches!(
                do_form(load!("(do true)"), env.clone()),
                Ok(MalType::Bool(true))
            ));
            assert!(matches!(
                do_form(load!("(do (def! a 1) 2)"), env.clone()),
                Ok(MalType::Int(2))
            ));
            assert!(matches!(env_get(&env.clone(), "a"), Ok(MalType::Int(1))));
        }*/

        #[test]
        fn _if_form() {
            let env = _env_empty();
            assert!(matches!(
                if_form(load!("(if)"), env.clone()),
                Err(e) if !e.is_recoverable()));
            assert!(matches!(
                if_form(load!("(if 1)"), env.clone()),
                Err(e) if !e.is_recoverable()));
            assert!(matches!(
                if_form(load!("(if 1 2 3 4)"), env.clone()),
                Err(e) if !e.is_recoverable()));
            assert!(matches!(
                if_form(load!("(if nil 1)"), env.clone()),
                Ok(MalType::Nil)
            ));
            assert!(matches!(
                if_form(load!("(if nil 1 2)"), env.clone()),
                Ok(MalType::Int(2))
            ));
            assert!(matches!(
                if_form(load!("(if true 1)"), env.clone()),
                Ok(MalType::Int(1))
            ));
            assert!(matches!(
                if_form(load!("(if true 1 2)"), env.clone()),
                Ok(MalType::Int(1))
            ));
        }

        #[test]
        fn fn_star() {
            let env = _env_empty();
            assert!(matches!(
                fn_star_form(load!("(fn* (a b) 1 2)"), env.clone()),
                Ok(MalType::MalFun {params, ast, .. })
                if matches!((*params).clone(), MalType::List(v)
                    if matches!(&v[0], MalType::Sym(v) if v == "a")
                    && matches!(&v[1], MalType::Sym(v) if v == "b")
                && matches!((*ast).clone(), MalType::List(v)
                    if matches!(&v[0], MalType::Int(1))
                    && matches!(&v[1], MalType::Int(2))))));
            // We trust the fact that the env does not do silly stuff
            assert!(matches!(
                fn_star_form(load!("(fn*)"), env.clone()),
                Err(e) if !e.is_recoverable()));
            assert!(matches!(
                fn_star_form(load!("(fn* 1)"), env.clone()),
                Err(e) if !e.is_recoverable()));
        }

        /*
        #[test]
        fn _eval_func() {
            let env = _env_empty();
            assert!(matches!(
                def_bang_form(load!("(def! or (fn* (a b) (if a a b)))"), env.clone()),
                Ok(_)
            ));
            assert!(matches!(
                eval_func(&MalType::Int(1)),
                Err(e) if !e.is_recoverable()));
            assert!(matches!(
                eval_func(load_f!("()", env.clone())),
                Err(e) if !e.is_recoverable()));
            assert!(matches!(
                eval_func(load_f!("(or nil nil)", env.clone())),
                Ok(v) if matches!(v, MalType::Nil)));
            assert!(matches!(
                eval_func(load_f!("(or 1 nil)", env.clone())),
                Ok(MalType::Int(1))
            ));
            assert!(matches!(
                eval_func(load_f!("(or nil 1)", env.clone())),
                Ok(MalType::Int(1))
            ));
        }*/

        /*#[test]
        fn _apply() {
            let env = _env_empty();
            assert!(matches!(
                apply(load2!("(def! a 1)"), env.clone()),
                Ok(MalType::Int(1))
            ));
            assert!(matches!(env_get(&env, "a"), Ok(MalType::Int(1))));
        }*/
    }
}
