use std::rc::Rc;

use crate::types::MalType as M;
use crate::types::{escape_str, MalType};

fn key_str(val: &str) -> MalType {
    if val.starts_with('ʞ') {
        M::Key(val.into())
    } else {
        M::Str(val.into())
    }
}

pub fn pr_str(ast: &MalType, print_readably: bool) -> String {
    match ast {
        M::Nil => "NIL".to_string(),
        M::Sym(sym) => sym.to_string(),
        M::Key(sym) => sym[2..].to_string(),
        M::Int(val) => val.to_string(),
        M::Bool(val) => val.to_string(),
        M::Str(str) => {
            if print_readably {
                escape_str(str)
            } else {
                str.to_string()
            }
        }
        M::List(el) => format!(
            "({})",
            el.iter()
                .map(|e| pr_str(e, print_readably))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        // This is truly horrible
        M::Vector(el) => format!(
            "[{}]",
            el.iter()
                .map(|e| pr_str(e, print_readably))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        M::Map(el) => format!(
            "{{{}}}",
            el.iter()
                .map(|sub| [
                    pr_str(&key_str(sub.0), print_readably),
                    pr_str(sub.1, print_readably)
                ]
                .join(" "))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        M::Fun(..) => "#<builtin>".to_string(),
        M::MalFun { .. } => "#<function>".to_string(),
    }
}

pub fn prt(ast: &MalType) -> String {
    pr_str(ast, true)
}

pub fn print_malfun(sym: &str, params: Rc<MalType>, ast: Rc<MalType>) {
    println!("{}\t[function]: {}", sym, prt(&params));
    ast.as_ref()
        .if_list()
        .unwrap_or(&[])
        .iter()
        .for_each(|el| println!(";   {}", pr_str(el, true)));
    println!();
}
