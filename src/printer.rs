use types::MalType;
use types::MalEnum;

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

    match *t.val.borrow() {
        MalEnum::Nil => "nil".to_string(),
        MalEnum::Int(x) => x.to_string(),
        MalEnum::Float(f) => f.to_string(),
        MalEnum::Bool(b) => b.to_string(),
        MalEnum::Str(s) => if print_readably {
            escape(&s)
        } else {
            s.to_string()
        },
        MalEnum::Symbol(s) => s.to_string(),
        MalEnum::KeyWord(s) => s.to_string(),
        MalEnum::Atom(s) => {
            let mut result = String::new();
            result.push_str("(atom ");

            result.push_str(&pr_str(&s, print_readably));
            result.push_str(")");
            result
        }
        MalEnum::List(l) => {
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
        MalEnum::Vector(l) => {
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
        MalEnum::Map(l) => {
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
        MalEnum::Error(s) => s.to_string(),
        MalEnum::Func(_, is_macro) => format!("#<function>: is_macro({})", is_macro),
        MalEnum::TCOFunc(_, _, _, _, is_macro) => format!("#<functionTCO>: is_macro({})", is_macro),
    }
}
