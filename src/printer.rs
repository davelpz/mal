use types::MalType;

fn escape(s: &str) -> String {
    let len = s.len();

    if len > 0 {
        let mut result = "\"".to_string();
        let fixed = s
            .replace("\\", "\\\\")
            .replace("\n", "\\n")
            .replace("\t", "\\t")
            .replace("\"", "\\\"");
        result.push_str(&fixed);
        result.push('"');

        result
    } else {
        s.to_string()
    }
}

pub fn pr_str(t: &MalType, print_readably: bool) -> String {
    //println!("{:?}",t);

    match t {
        MalType::Nil => "nil".to_string(),
        MalType::Int(x) => x.to_string(),
        MalType::Float(f) => f.to_string(),
        MalType::Bool(b) => b.to_string(),
        MalType::Str(s) => if print_readably { escape(s) } else { s.to_string() },
        MalType::Symbol(s) => s.to_string(),
        MalType::KeyWord(s) => s.to_string(),
        MalType::List(l) => {
            let mut result = String::new();
            result.push_str("(");

            for (i, item) in l.iter().enumerate() {
                if i > 0 {
                    result.push_str(" ");
                }
                result.push_str(&pr_str(item, print_readably));
            }

            result.push_str(")");
            result
        }
        MalType::Vector(l) => {
            let mut result = String::new();
            result.push_str("[");

            for (i, item) in l.iter().enumerate() {
                if i > 0 {
                    result.push_str(" ");
                }
                result.push_str(&pr_str(item, print_readably));
            }

            result.push_str("]");
            result
        }
        MalType::Map(l) => {
            let mut result = String::new();
            result.push_str("{");

            for (i, item) in l.iter().enumerate() {
                if i > 0 {
                    result.push_str(" ");
                }
                result.push_str(&pr_str(item, print_readably));
            }

            result.push_str("}");
            result
        }
        MalType::Error(s) => s.to_string(),
        MalType::Func(_) => "#<function>".to_string(),
    }
}
