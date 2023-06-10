use crate::envs::Env;
use crate::types::MalType::*;
use crate::types::{MalRet, MalType};

fn function_call(list: MalType) -> MalRet {
    match list {
        List(list) => {
            let _func = &list[0];
            if list.len() > 1 {
                let _ast = &list[1..list.len() - 1];
                todo!("call: func(args)");
            } else {
                todo!("call: func()");
            }
        }
        _ => Err("YOU SHOULD NOT BE HERE".to_string()),
    }
}

pub fn eval(ast: MalType, env: &Env) -> MalRet {
    match &ast {
        List(list) => {
            if list.is_empty() {
                Ok(ast)
            } else {
                let ev = eval_ast(ast, env)?;
                function_call(ev)
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

pub fn eval_ast(ast: MalType, env: &Env) -> MalRet {
    match ast {
        Sym(sym) => env.solve(sym), // resolve
        List(list) => eval_list(list, env),
        _ => Ok(ast), // Default behavior, do not resolve
    }
}
