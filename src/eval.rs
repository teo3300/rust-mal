use crate::env::Env;
use crate::types::MalType::*;
use crate::types::{MalArgs, MalMap, MalRet, MalType};

fn call_func(func: &MalType, args: &[MalType]) -> MalRet {
    match func {
        Fun(func) => func(args),
        _ => Err(format!("{:?} is not a function", func)),
    }
}

fn eval_func(list: &MalType) -> MalRet {
    match list {
        List(list) => {
            let (func, args) = car_cdr(list);
            call_func(func, args)
        }
        _ => todo!("Yep! I hate it"),
    }
}

fn car_cdr(list: &[MalType]) -> (&MalType, &[MalType]) {
    (
        &list[0],
        if list.len() > 1 {
            &list[1..]
        } else {
            &list[0..0]
        },
    )
}

fn def_bang(list: &[MalType], env: &mut Env) -> MalRet {
    match list.len() {
        2 => match &list[0] {
            Sym(sym) => {
                let cdr = eval(&list[1], env)?;
                env.set(sym.as_str(), &cdr);
                Ok(cdr)
            }
            _ => Err(format!(
                "Assigning {:?} to {:?}, which is not a symbol",
                &list[1], &list[0]
            )),
        },
        _ => Err("def! macro needs 2 arguments".to_string()),
    }
}

fn let_star(list: &[MalType], env: &Env) -> MalRet {
    // Create the inner environment
    let mut inner_env = Env::new(Some(Box::new(env.clone())));
    // change the inner environment
    let (car, cdr) = car_cdr(list);
    match car {
        List(list) if list.len() % 2 == 0 => {
            // TODO: Find a way to avoid index looping that is ugly
            for i in (0..list.len()).step_by(2) {
                match &list[i] {
                    Sym(_) => def_bang(&list[i..i + 2], &mut inner_env)?,
                    _ => return Err(format!("Map key not valid: {:?}", list[i])),
                };
            }
            eval(&cdr[0], &mut inner_env)
        }
        _ => Err("First argument of let* must be an even-length list".to_string()),
    }
}

fn apply(list: &MalArgs, env: &mut Env) -> MalRet {
    let (car, cdr) = car_cdr(list);
    match car {
        Sym(sym) if sym == "def!" => def_bang(cdr, env), // already remove the def
        Sym(sym) if sym == "let*" => let_star(cdr, env),
        Sym(_) => eval_func(&eval_ast(&List(list.to_vec()), env)?),
        _ => Err("First element not a symbol".to_string()),
    }
}

pub fn eval(ast: &MalType, env: &mut Env) -> MalRet {
    match &ast {
        List(list) if list.is_empty() => Ok(ast.clone()),
        List(list) if !list.is_empty() => apply(list, env),
        _ => eval_ast(ast, env),
    }
}

fn eval_collection(list: &MalArgs, env: &mut Env) -> Result<MalArgs, String> {
    let mut ret = MalArgs::new();
    for el in list {
        match eval(el, env) {
            Ok(val) => ret.push(val),
            Err(err) => return Err(err),
        }
    }
    Ok(ret)
}

fn eval_map(map: &MalMap, env: &mut Env) -> MalRet {
    let mut ret = MalMap::new();

    for (k, v) in map {
        match eval(v, env) {
            Ok(res) => ret.insert(k.to_string(), res),
            Err(err) => return Err(err),
        };
    }

    Ok(Map(ret))
}

fn eval_ast(ast: &MalType, env: &mut Env) -> MalRet {
    match ast {
        Sym(sym) => env.get(sym),
        List(list) => Ok(List(eval_collection(list, env)?)),
        Vector(vec) => Ok(Vector(eval_collection(vec, env)?)),
        Map(map) => eval_map(map, env),
        _ => Ok(ast.clone()),
    }
}
