use crate::types::escape_str;
use crate::types::MalType;
use crate::types::MalType::*;

pub fn pr_str(ast: &MalType, print_readably: bool) -> String {
    match ast {
        Nil => "nil".to_string(),
        Sym(sym) => sym.to_string(),
        Key(sym) => sym[1..sym.len() - 1].to_string(),
        Int(val) => val.to_string(),
        Bool(val) => val.to_string(),
        Str(str) => {
            if print_readably {
                escape_str(str)
            } else {
                str.to_string()
            }
        }
        List(el) => format!(
            "({})",
            el.iter()
                .map(|e| pr_str(e, print_readably))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        // This is truly horrible
        Vector(el) => format!(
            "[{}]",
            el.iter()
                .map(|e| pr_str(e, print_readably))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        Map(el) => format!(
            "{{{}}}",
            el.iter()
                .map(|sub| vec![sub.0.to_string(), pr_str(sub.1, print_readably)].join(" "))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        Fun(func) => format!("{:?}", func),
    }
}
