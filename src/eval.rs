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
            let func = &list[0];
            let args = if list.len() > 1 {
                &list[1..]
            } else {
                &list[0..0]
            };
            call_func(func, args)
        }
        _ => Err("YOU SHOULD NOT BE HERE".to_string()),
    }
}

pub fn eval(ast: &MalType, env: &Env) -> MalRet {
    match &ast {
        List(list) => {
            if list.is_empty() {
                // Ok(Nil) // Should be the normal behavior
                Ok(ast.clone())
            } else {
                eval_func(&eval_ast(ast, env)?)
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn eval_collection(list: &MalArgs, env: &Env) -> Result<MalArgs, String> {
    let mut ret = MalArgs::new();
    for el in list {
        match eval(el, env) {
            Ok(val) => ret.push(val),
            Err(err) => return Err(err),
        }
    }
    Ok(ret)
}

fn eval_map(map: &MalMap, env: &Env) -> MalRet {
    let mut ret = MalMap::new();

    for (k, v) in map {
        match eval(v, env) {
            Ok(res) => ret.insert(k.to_string(), res),
            Err(err) => return Err(err),
        };
    }

    Ok(Map(ret))
}

fn eval_ast(ast: &MalType, env: &Env) -> MalRet {
    match ast {
        Sym(sym) => env.get(sym),
        List(list) => Ok(List(eval_collection(list, env)?)),
        Vector(vec) => Ok(Vector(eval_collection(vec, env)?)),
        Map(map) => eval_map(map, env),
        _ => Ok(ast.clone()),
    }
}
