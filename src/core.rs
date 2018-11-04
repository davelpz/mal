use eval::Environment;
use std::rc::Rc;
use types::BuiltinFunc;
use types::BuiltinFuncArgs;
use types::MalType;

pub fn create_namespace() -> Vec<(&'static str, Rc<Box<BuiltinFunc>>)> {
    let mut ns: Vec<(&str, Rc<Box<BuiltinFunc>>)> = Vec::new();

    ns.push(("+", Rc::new(Box::new(addition_builtin))));
    ns.push(("-", Rc::new(Box::new(subtraction_builtin))));
    ns.push(("*", Rc::new(Box::new(multiplication_builtin))));
    ns.push(("/", Rc::new(Box::new(division_builtin))));

    ns
}

pub fn init_environment(env: &mut Environment) {
    for tup in create_namespace() {
        env.set(tup.0.to_string(),MalType::Func(tup.1));
    }
    /*
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
    */
}

fn all_numeric(args: &BuiltinFuncArgs) -> bool {
    args.iter().all(|i| match i {
        MalType::Int(_) | MalType::Float(_) => true,
        _ => false,
    })
}

fn all_int(args: &BuiltinFuncArgs) -> bool {
    args.iter().all(|i| match i {
        MalType::Int(_) => true,
        _ => false,
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
