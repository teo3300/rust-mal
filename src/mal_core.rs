use crate::envs::Env;
use crate::types::MalType::*;
use crate::types::{MalArgs, MalRet, MalType};

fn call_func(func: &MalType, args: MalArgs) -> MalRet {
    match func {
        Fun(func) => func(args),
        _ => Err(format!("{:?} is not a function", func)),
    }
}

fn eval_func(list: MalType) -> MalRet {
    match list {
        List(list) => {
            let func = &list[0];
            let args = if list.len() > 1 {
                &list[1..]
            } else {
                &list[0..0]
            };
            call_func(func, args.to_vec())
        }
        _ => Err("YOU SHOULD NOT BE HERE".to_string()),
    }
}

pub fn eval(ast: MalType, env: &Env) -> MalRet {
    match &ast {
        List(list) => {
            if list.is_empty() {
                // Ok(Nil) // Should be the normal behavior
                Ok(ast)
            } else {
                eval_func(eval_ast(ast, env)?)
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn eval_list(list: Vec<MalType>, env: &Env) -> MalRet {
    let mut ret = Vec::new();
    for el in list {
        match eval(el, env) {
            Ok(val) => ret.push(val),
            Err(err) => return Err(err),
        }
    }
    Ok(List(ret))
}

fn eval_ast(ast: MalType, env: &Env) -> MalRet {
    match ast {
        Sym(sym) => env.solve(sym),
        List(list) => eval_list(list, env),
        _ => Ok(ast),
    }
}
