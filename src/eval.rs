use printer::pr_str;
use std::collections::HashMap;
use std::rc::Rc;
use types::BuiltinFuncArgs;
use types::MalType;

//Defining Environment type for mal
#[derive(Debug, Clone)]
pub struct Environment<'a> {
    pub data: HashMap<String, MalType>,
    pub outer: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            data: HashMap::new(),
            outer: None,
        }
    }

    pub fn set(&mut self, key: String, value: MalType) -> MalType {
        let c = value.clone();
        self.data.insert(key, value);
        c
    }

    pub fn find(&self, key: String) -> Option<MalType> {
        match self.data.get(&key) {
            Some(v) => Some(v.clone()),
            None => match self.outer {
                None => None,
                Some(ref p) => p.find(key),
            },
        }
    }

    pub fn get(&self, key: String) -> MalType {
        match self.find(key.clone()) {
            Some(v) => v,
            None => MalType::Error(format!("{} not found.", key)),
        }
    }

    pub fn get_inner(&mut self) -> Environment {
        let mut inner = Environment::new();
        inner.outer = Some(self);
        inner
    }
}

pub fn init_environment(env: &mut Environment) {
    env.set(
        "+".to_string(),
        MalType::Func(Rc::new(Box::new(addition_builtin))),
    );
    env.set(
        "-".to_string(),
        MalType::Func(Rc::new(Box::new(subtraction_builtin))),
    );
    env.set(
        "*".to_string(),
        MalType::Func(Rc::new(Box::new(multiplication_builtin))),
    );
    env.set(
        "/".to_string(),
        MalType::Func(Rc::new(Box::new(division_builtin))),
    );
}

fn all_numeric(args: &BuiltinFuncArgs) -> bool {
    args.iter().all(|i| match i {
        MalType::Int(_) | MalType::Float(_) => true,
        _ => return false,
    })
}

fn all_int(args: &BuiltinFuncArgs) -> bool {
    args.iter().all(|i| match i {
        MalType::Int(_) => true,
        _ => return false,
    })
}

fn addition_builtin(args: BuiltinFuncArgs) -> MalType {
    //Check to make sure we have only numeric types
    if !all_numeric(&args) {
        return MalType::Error("Wrong types for +".to_string());
    }

    if all_int(&args) {
        let mut result: i64 = 0;
        for i in args {
            result += match i {
                MalType::Int(x) => x,
                _ => 0,
            }
        }
        MalType::Int(result)
    } else {
        let mut result: f64 = 0.0;
        for i in args {
            result += match i {
                MalType::Float(x) => x,
                MalType::Int(y) => y as f64,
                _ => 0.0,
            }
        }
        MalType::Float(result)
    }
}

fn subtraction_builtin(args: BuiltinFuncArgs) -> MalType {
    //Check to make sure we have only numeric types
    if !all_numeric(&args) {
        return MalType::Error("Wrong types for -".to_string());
    }

    if all_int(&args) {
        let mut result: i64 = args[0].get_int();
        for i in args.iter().skip(1) {
            result -= match i {
                MalType::Int(x) => *x,
                _ => 0,
            }
        }
        MalType::Int(result)
    } else {
        let mut result: f64 = args[0].get_float();
        for i in args.iter().skip(1) {
            result -= match i {
                MalType::Float(x) => *x,
                MalType::Int(y) => *y as f64,
                _ => 0.0,
            }
        }
        MalType::Float(result)
    }
}

fn multiplication_builtin(args: BuiltinFuncArgs) -> MalType {
    //Check to make sure we have only numeric types
    if !all_numeric(&args) {
        return MalType::Error("Wrong types for *".to_string());
    }

    if all_int(&args) {
        let mut result: i64 = args[0].get_int();
        for i in args.iter().skip(1) {
            result *= match i {
                MalType::Int(x) => *x,
                _ => 0,
            }
        }
        MalType::Int(result)
    } else {
        let mut result: f64 = args[0].get_float();
        for i in args.iter().skip(1) {
            result *= match i {
                MalType::Float(x) => *x,
                MalType::Int(y) => *y as f64,
                _ => 0.0,
            }
        }
        MalType::Float(result)
    }
}

fn division_builtin(args: BuiltinFuncArgs) -> MalType {
    //Check to make sure we have only numeric types
    if !all_numeric(&args) {
        return MalType::Error("Wrong types for /".to_string());
    }

    if all_int(&args) {
        let mut result: i64 = args[0].get_int();
        for i in args.iter().skip(1) {
            result /= match i {
                MalType::Int(x) => *x,
                _ => 0,
            }
        }
        MalType::Int(result)
    } else {
        let mut result: f64 = args[0].get_float();
        for i in args.iter().skip(1) {
            result /= match i {
                MalType::Float(x) => *x,
                MalType::Int(y) => *y as f64,
                _ => 0.0,
            }
        }
        MalType::Float(result)
    }
}

pub fn eval<'a, 'b>(t: &'a MalType, env: &'b mut Environment) -> MalType {
    match t {
        MalType::List(l) if l.len() == 0 => t.clone(),
        MalType::List(l1) if l1.len() > 0 => {
            let eval_list = eval_ast(t, env);
            if let MalType::List(l2) = eval_list {
                let first = &l2[0];
                if let MalType::Symbol(s) = first {
                    if s == "def!" {
                        let second = &l1[1];
                        let third = eval(&l2[2], env);
                        //println!("{:?}",second);
                        //println!("{:?}",third);
                        env.set(second.get_symbol_string(), third)
                    } else if s == "let*" {
                        MalType::Nil
                    } else {
                        MalType::Nil
                    }
                } else if let MalType::Func(f) = first {
                    f(l2[1..].to_vec())
                } else if let MalType::Error(_) = first {
                    MalType::Error(format!("{}", pr_str(first)))
                } else {
                    MalType::Error(format!("{} not found.", pr_str(first)))
                }
            } else {
                MalType::Error("internal error: eval_ast of List did not return a List".to_string())
            }
        }
        _ => eval_ast(t, env),
    }
}

pub fn eval_ast<'a, 'b>(t: &'a MalType, env: &'b mut Environment) -> MalType {
    match t {
        MalType::Symbol(s) if s == "def!" => t.clone(),
        MalType::Symbol(s) if s == "let*" => t.clone(),
        MalType::Symbol(s) => {
            let lookup = env.get(s.to_string());
            match lookup {
                MalType::Func(f) => MalType::Func(f.clone()),
                MalType::Error(_) => MalType::Error(format!("{} not found.", s)),
                _ => lookup,
            }
        }
        MalType::List(l) => {
            let new_l: Vec<MalType> = l.iter().map(|item| eval(item, env)).collect();
            MalType::List(new_l)
        }
        MalType::Vector(l) => {
            let new_l: Vec<MalType> = l.iter().map(|item| eval(item, env)).collect();
            MalType::Vector(new_l)
        }
        MalType::Map(l) => {
            let new_l: Vec<MalType> = l
                .iter()
                .enumerate()
                .map(|tup| {
                    if tup.0 % 2 == 1 {
                        eval(tup.1, env)
                    } else {
                        tup.1.clone()
                    }
                }).collect();
            MalType::Map(new_l)
        }
        _ => t.clone(),
    }
}

/*
  Unit Tests for various functions/methods
*/
#[cfg(test)]
mod tests {
    use super::*;
    use reader::read_str;

    #[test]
    fn eval_test() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let ast = read_str("(+ 1 2)");
        assert_eq!(eval(&ast, &mut env), MalType::Int(3));

        let ast = read_str("(+ 5 (* 2 3))");
        assert_eq!(eval(&ast, &mut env), MalType::Int(11));

        let ast = read_str("(- (+ 5 (* 2 3)) 3)");
        assert_eq!(eval(&ast, &mut env), MalType::Int(8));

        let ast = read_str("(/ (- (+ 5 (* 2 3)) 3) 4)");
        assert_eq!(eval(&ast, &mut env), MalType::Int(2));

        let ast = read_str("(/ (- (+ 515 (* 87 311)) 302) 27)");
        assert_eq!(eval(&ast, &mut env), MalType::Int(1010));

        let ast = read_str("(* -3 6)");
        assert_eq!(eval(&ast, &mut env), MalType::Int(-18));

        let ast = read_str("(/ (- (+ 515 (* -87 311)) 296) 27)");
        assert_eq!(eval(&ast, &mut env), MalType::Int(-994));

        let ast = read_str("(abc 1 2 3)");
        assert_eq!(
            eval(&ast, &mut env),
            MalType::Error("abc not found.".to_string())
        );

        let ast = read_str("()");
        let empty_vec: Vec<MalType> = vec![];
        assert_eq!(eval(&ast, &mut env), MalType::List(empty_vec));

        let ast = read_str("[1 2 (+ 1 2)]");
        let result_vec: Vec<MalType> = vec![MalType::Int(1), MalType::Int(2), MalType::Int(3)];
        assert_eq!(eval(&ast, &mut env), MalType::Vector(result_vec));

        let ast = read_str("{\"a\" (+ 7 8)}");
        let result_vec: Vec<MalType> = vec![MalType::Str("\"a\"".to_string()), MalType::Int(15)];
        assert_eq!(eval(&ast, &mut env), MalType::Map(result_vec));

        let ast = read_str("{:a (+ 7 8)}");
        let result_vec: Vec<MalType> = vec![MalType::KeyWord(":a".to_string()), MalType::Int(15)];
        assert_eq!(eval(&ast, &mut env), MalType::Map(result_vec));
    }
}
