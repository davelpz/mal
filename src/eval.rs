use std::collections::HashMap;
use std::rc::Rc;
use types::BuiltinFuncArgs;
use types::Env;
use types::MalError;
use types::MalType;

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
                } else {
                    MalType::Error(
                        "internal error: First element of List is not a function".to_string(),
                    )
                }
            } else {
                MalType::Error(
                    "internal error: eval_ast of List did not return a List".to_string(),
                )
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
                None =>  MalType::Error(format!("{} not defined.", s)),
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
            let new_l: Vec<MalType> = l.iter().enumerate().map(|tup| {
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
