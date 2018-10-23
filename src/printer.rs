use types::MalType;


pub fn pr_str(t: &MalType) -> String {
    println!("{:?}",t);
    
    match t {
        MalType::Nil => "Nil".to_string(),
        MalType::Int(x) => x.to_string(),
        MalType::Float(f) => f.to_string(),
        MalType::Str(s) => s.to_string(),
        MalType::Symbol(s) => s.to_string(),
        MalType::List(l) => {
            let mut result = String::new();
            result.push_str("(");

            for i in l {
                result.push_str(&pr_str(i));
                result.push_str(" ");
            }

            result.push_str(")");
            result
        }
    }
}