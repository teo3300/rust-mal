use crate::types::MalType;

pub fn pr_str(ast: &MalType) -> String {
    match ast {
        MalType::Nil => "nil".to_string(),
        MalType::Symbol(sym) => sym.to_string(),
        MalType::Integer(val) => val.to_string(),
        MalType::Bool(val) => val.to_string(),
        MalType::List(el) => format!(
            "({})",
            el.iter()
                .map(|sub| pr_str(sub))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        // This is truly horrible
        MalType::Vector(el) => format!(
            "[{}]",
            el.iter()
                .map(|sub| pr_str(sub))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        MalType::Map(el) => format!(
            "{{{}}}",
            el.iter()
                .map(|sub| vec![pr_str(sub.0), pr_str(sub.1)].join(" "))
                .collect::<Vec<String>>()
                .join(" ")
        ),
    }
}
