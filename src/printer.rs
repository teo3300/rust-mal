use crate::types::MalType;

pub fn pr_str(ast: &MalType) -> String {
    match ast {
        MalType::Symbol(sym) => sym.to_string(),
        MalType::Integer(val) => val.to_string(),
        MalType::List(el) => format!(
            "({})",
            el.iter()
                .map(|sub| pr_str(sub))
                .collect::<Vec<String>>()
                .join(" ")
        ),
    }
}
