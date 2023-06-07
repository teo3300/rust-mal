use crate::types::escape_str;
use crate::types::MalType;
use crate::types::MalType::*;

pub fn pr_str(ast: &MalType, print_readably: bool) -> String {
    match ast {
        Nil => "nil".to_string(),
        Sym(sym) | Key(sym) => sym.val.to_string(),
        Int(val) => val.to_string(),
        Bool(val) => val.to_string(),
        Str(str) => {
            if print_readably {
                escape_str(&str.val)
            } else {
                str.val.to_string()
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
                .map(|sub| vec![sub.0.val.to_string(), pr_str(sub.1, print_readably)].join(" "))
                .collect::<Vec<String>>()
                .join(" ")
        ),
    }
}
