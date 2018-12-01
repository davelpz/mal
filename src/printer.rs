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

    if t.is_nil() {
        "nil".to_string()
    } else if t.is_int() {
        t.get_int().to_string()
    } else if t.is_float() {
        t.get_float().to_string()
    } else if t.is_bool() {
        t.get_bool().to_string()
    } else if t.is_string() {
        let s = t.get_string();
        if print_readably {
            escape(&s)
        } else {
            s.to_string()
        }
    } else if t.is_symbol() {
        t.get_string()
    } else if t.is_keyword() {
        t.get_string()
    } else if t.is_atom() {
        let s = t.get_atom();
        let mut result = String::new();
        result.push_str("(atom ");

        result.push_str(&pr_str(&s, print_readably));
        result.push_str(")");
        result
    } else if t.is_list() {
        let l = t.get_list();
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
    } else if t.is_vector() {
        let l = t.get_list();
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
    } else if t.is_map() {
        let l = t.get_list();
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
    } else if t.is_error() {
        t.get_string()
    } else if t.is_func() {
        format!("#<function>: is_macro({})", t.is_macro())
    } else if t.is_func_tco() {
        format!("#<functionTCO>: is_macro({})", t.is_macro())
    } else {
        "".to_string()
    }
}
