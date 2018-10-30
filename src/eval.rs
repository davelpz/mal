use std::collections::HashMap;
use std::rc::Rc;
use types::BuiltinFuncArgs;
use types::Env;
use types::MalType;
use printer::pr_str;

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

pub fn init_repl_env() -> Env {
    let mut repl_env: Env = HashMap::new();

    repl_env.insert("+".to_string(), Rc::new(Box::new(addition_builtin)));
    repl_env.insert("-".to_string(), Rc::new(Box::new(subtraction_builtin)));
    repl_env.insert("*".to_string(), Rc::new(Box::new(multiplication_builtin)));
    repl_env.insert("/".to_string(), Rc::new(Box::new(division_builtin)));

    repl_env
}

pub fn eval<'a, 'b>(t: &'a MalType, env: &'b Env) -> MalType {
    match t {
        MalType::List(l) if l.len() == 0 => t.clone(),
        MalType::List(l) if l.len() > 0 => {
            let eval_list = eval_ast(t, env);
            if let MalType::List(l) = eval_list {
                let first = &l[0];
                if let MalType::Func(f) = first {
                    f(l[1..].to_vec())
                } else if let MalType::Error(_) = first {    
                    MalType::Error(
                        format!("{}", pr_str(first))
                    )
                } else {
                    MalType::Error(
                        format!("{} not found.", pr_str(first))
                    )
                }
            } else {
                MalType::Error("internal error: eval_ast of List did not return a List".to_string())
            }
        }
        _ => eval_ast(t, env),
    }
}

pub fn eval_ast<'a, 'b>(t: &'a MalType, env: &'b Env) -> MalType {
    match t {
        MalType::Symbol(s) => {
            let lookup = env.get(s);
            match lookup {
                Some(f) => MalType::Func(f.clone()),
                None => MalType::Error(format!("{} not found.", s)),
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
    use eval::init_repl_env;
    use reader::read_str;

    #[test]
    fn eval_test() {
        let env = init_repl_env();

        let ast = read_str("(+ 1 2)");
        assert_eq!(eval(&ast, &env), MalType::Int(3));

        let ast = read_str("(+ 5 (* 2 3))");
        assert_eq!(eval(&ast, &env), MalType::Int(11));

        let ast = read_str("(- (+ 5 (* 2 3)) 3)");
        assert_eq!(eval(&ast, &env), MalType::Int(8));

        let ast = read_str("(/ (- (+ 5 (* 2 3)) 3) 4)");
        assert_eq!(eval(&ast, &env), MalType::Int(2));

        let ast = read_str("(/ (- (+ 515 (* 87 311)) 302) 27)");
        assert_eq!(eval(&ast, &env), MalType::Int(1010));

        let ast = read_str("(* -3 6)");
        assert_eq!(eval(&ast, &env), MalType::Int(-18));

        let ast = read_str("(/ (- (+ 515 (* -87 311)) 296) 27)");
        assert_eq!(eval(&ast, &env), MalType::Int(-994));

        let ast = read_str("(abc 1 2 3)");
        assert_eq!(eval(&ast, &env), MalType::Error("abc not found.".to_string()));

        let ast = read_str("()");
        let empty_vec : Vec<MalType> = vec![];
        assert_eq!(eval(&ast, &env), MalType::List(empty_vec));

        let ast = read_str("[1 2 (+ 1 2)]");
        let result_vec : Vec<MalType> = vec![MalType::Int(1), MalType::Int(2), MalType::Int(3)];
        assert_eq!(eval(&ast, &env), MalType::Vector(result_vec));

        let ast = read_str("{\"a\" (+ 7 8)}");
        let result_vec : Vec<MalType> = vec![MalType::Str("\"a\"".to_string()), MalType::Int(15)];
        assert_eq!(eval(&ast, &env), MalType::Map(result_vec));

        let ast = read_str("{:a (+ 7 8)}");
        let result_vec : Vec<MalType> = vec![MalType::KeyWord(":a".to_string()), MalType::Int(15)];
        assert_eq!(eval(&ast, &env), MalType::Map(result_vec));
    }
}
