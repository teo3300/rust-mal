use std::rc::Rc;

use crate::types::escape_str;
use crate::types::MalType;
use crate::types::MalType as M;

pub fn pr_str(ast: &MalType, print_readably: bool) -> String {
    match ast {
        M::Nil => "nil".to_string(),
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
                .map(|sub| [sub.0.to_string(), pr_str(sub.1, print_readably)].join(" "))
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
        .unwrap_or_else(|_| &[])
        .iter()
        .for_each(|el| println!(";   {}", prt(el)));
}
