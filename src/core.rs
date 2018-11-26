use eval::Environment;
use printer;
use reader::read_str;
use rep;
use std::cell::RefCell;
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
    ns.push(("prn", Rc::new(Box::new(prn_builtin))));
    ns.push(("println", Rc::new(Box::new(println_builtin))));
    ns.push(("pr-str", Rc::new(Box::new(pr_str_builtin))));
    ns.push(("str", Rc::new(Box::new(str_builtin))));
    ns.push(("list", Rc::new(Box::new(list_builtin))));
    ns.push(("list?", Rc::new(Box::new(list_test_builtin))));
    ns.push(("empty?", Rc::new(Box::new(empty_test_builtin))));
    ns.push(("count", Rc::new(Box::new(count_builtin))));
    ns.push(("=", Rc::new(Box::new(equals_builtin))));
    ns.push(("<", Rc::new(Box::new(lt_builtin))));
    ns.push(("<=", Rc::new(Box::new(le_builtin))));
    ns.push((">", Rc::new(Box::new(gt_builtin))));
    ns.push((">=", Rc::new(Box::new(ge_builtin))));
    ns.push(("read-string", Rc::new(Box::new(read_string_builtin))));
    ns.push(("slurp", Rc::new(Box::new(slurp_builtin))));
    ns.push(("atom", Rc::new(Box::new(atom_builtin))));
    ns.push(("atom?", Rc::new(Box::new(atom_test_builtin))));
    ns.push(("deref", Rc::new(Box::new(deref_builtin))));
    ns.push(("reset!", Rc::new(Box::new(reset_builtin))));
    ns.push(("swap!", Rc::new(Box::new(swap_builtin))));
    ns.push(("cons", Rc::new(Box::new(cons_builtin))));
    ns.push(("concat", Rc::new(Box::new(concat_builtin))));

    ns
}

pub fn init_environment(env: &mut Environment) {
    for tup in create_namespace() {
        env.set(tup.0.to_string(), MalType::Func(tup.1));
    }

    env.set("*ARGV*".to_string(), MalType::List(Vec::new()));

    rep("(def! not (fn* (a) (if a false true)))", env);
    rep(
        "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \")\")))))",
        env,
    );
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

fn prn_helper(args: BuiltinFuncArgs, print_readably: bool, delimiter: &str) -> String {
    let mut result = String::new();

    for (i, t) in args.iter().enumerate() {
        //println!("{} = {:?} ",i,t);
        if i > 0 {
            result.push_str(&delimiter);
        }
        result.push_str(&printer::pr_str(&t, print_readably));
        //println!("{}", result);
    }

    result
}

fn prn_builtin(args: BuiltinFuncArgs) -> MalType {
    println!("{}", prn_helper(args, true, " "));

    MalType::Nil
}

fn println_builtin(args: BuiltinFuncArgs) -> MalType {
    println!("{}", prn_helper(args, false, " "));

    MalType::Nil
}

fn pr_str_builtin(args: BuiltinFuncArgs) -> MalType {
    MalType::Str(format!("{}", prn_helper(args, true, " ")))
}

fn str_builtin(args: BuiltinFuncArgs) -> MalType {
    MalType::Str(format!("{}", prn_helper(args, false, "")))
}

fn list_builtin(args: BuiltinFuncArgs) -> MalType {
    MalType::List(args)
}

fn list_test_builtin(args: BuiltinFuncArgs) -> MalType {
    match args.get(0) {
        Some(MalType::List(_)) => MalType::Bool(true),
        _ => MalType::Bool(false),
    }
}

fn empty_test_builtin(args: BuiltinFuncArgs) -> MalType {
    match args.get(0) {
        Some(MalType::List(x)) if x.is_empty() => MalType::Bool(true),
        Some(MalType::Vector(x)) if x.is_empty() => MalType::Bool(true),
        _ => MalType::Bool(false),
    }
}

fn count_builtin(args: BuiltinFuncArgs) -> MalType {
    match args.get(0) {
        Some(MalType::List(x)) => MalType::Int(x.len() as i64),
        Some(MalType::Vector(x)) => MalType::Int(x.len() as i64),
        _ => MalType::Int(0),
    }
}

fn equals_builtin_helper(a: &MalType, b: &MalType) -> bool {
    //println!("equals_builtin_helper: {:?}   {:?}", a, b);

    match a {
        MalType::Bool(av) => match b {
            MalType::Bool(bv) if av == bv => true,
            _ => false,
        },
        MalType::Error(av) => match b {
            MalType::Error(bv) if av == bv => true,
            _ => false,
        },
        MalType::Float(av) => match b {
            MalType::Float(bv) if av == bv => true,
            _ => false,
        },
        MalType::Func(av) => match b {
            MalType::Func(bv) if av == bv => true,
            _ => false,
        },
        MalType::TCOFunc(av1, av2, av3, av4) => match b {
            MalType::TCOFunc(bv1, bv2, bv3, bv4)
                if av1 == bv1 && av2 == bv2 && av3 == bv3 && av4 == bv4 =>
            {
                true
            }
            _ => false,
        },
        MalType::Int(av) => match b {
            MalType::Int(bv) if av == bv => true,
            _ => false,
        },
        MalType::KeyWord(av) => match b {
            MalType::KeyWord(bv) if av == bv => true,
            _ => false,
        },
        MalType::Nil => match b {
            MalType::Nil => true,
            _ => false,
        },
        MalType::Str(av) => match b {
            MalType::Str(bv) if av == bv => true,
            _ => false,
        },
        MalType::Symbol(av) => match b {
            MalType::Symbol(bv) if av == bv => true,
            _ => false,
        },
        MalType::Atom(av) => match b {
            MalType::Atom(bv) => equals_builtin_helper(&av.borrow(), &bv.borrow()),
            _ => false,
        },
        MalType::List(av) => match b {
            MalType::List(bv) | MalType::Vector(bv) => {
                (av.len() == bv.len())
                    && (av.iter().zip(bv).all(|(x, y)| equals_builtin_helper(x, y)))
            }
            _ => false,
        },
        MalType::Vector(av) => match b {
            MalType::Vector(bv) | MalType::List(bv) => {
                (av.len() == bv.len())
                    && (av.iter().zip(bv).all(|(x, y)| equals_builtin_helper(x, y)))
            }
            _ => false,
        },
        MalType::Map(av) => match b {
            MalType::Map(bv) => {
                (av.len() == bv.len())
                    && (av.iter().zip(bv).all(|(x, y)| equals_builtin_helper(x, y)))
            }
            _ => false,
        },
    }
}

fn equals_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        MalType::Bool(equals_builtin_helper(&args[0], &args[1]))
    } else {
        MalType::Bool(false)
    }
}

fn lt_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        match args[0] {
            MalType::Int(x) => match args[1] {
                MalType::Int(y) if x < y => MalType::Bool(true),
                MalType::Float(y) if (x as f64) < y => MalType::Bool(true),
                _ => MalType::Bool(false),
            },
            MalType::Float(x) => match args[1] {
                MalType::Float(y) if x < y => MalType::Bool(true),
                MalType::Int(y) if x < (y as f64) => MalType::Bool(true),
                _ => MalType::Bool(false),
            },
            _ => MalType::Bool(false),
        }
    } else {
        MalType::Bool(false)
    }
}

fn le_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        match args[0] {
            MalType::Int(x) => match args[1] {
                MalType::Int(y) if x <= y => MalType::Bool(true),
                MalType::Float(y) if (x as f64) <= y => MalType::Bool(true),
                _ => MalType::Bool(false),
            },
            MalType::Float(x) => match args[1] {
                MalType::Float(y) if x <= y => MalType::Bool(true),
                MalType::Int(y) if x <= (y as f64) => MalType::Bool(true),
                _ => MalType::Bool(false),
            },
            _ => MalType::Bool(false),
        }
    } else {
        MalType::Bool(false)
    }
}

fn gt_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        match args[0] {
            MalType::Int(x) => match args[1] {
                MalType::Int(y) if x > y => MalType::Bool(true),
                MalType::Float(y) if (x as f64) > y => MalType::Bool(true),
                _ => MalType::Bool(false),
            },
            MalType::Float(x) => match args[1] {
                MalType::Float(y) if x > y => MalType::Bool(true),
                MalType::Int(y) if x > (y as f64) => MalType::Bool(true),
                _ => MalType::Bool(false),
            },
            _ => MalType::Bool(false),
        }
    } else {
        MalType::Bool(false)
    }
}

fn ge_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        match args[0] {
            MalType::Int(x) => match args[1] {
                MalType::Int(y) if x >= y => MalType::Bool(true),
                MalType::Float(y) if (x as f64) >= y => MalType::Bool(true),
                _ => MalType::Bool(false),
            },
            MalType::Float(x) => match args[1] {
                MalType::Float(y) if x >= y => MalType::Bool(true),
                MalType::Int(y) if x >= (y as f64) => MalType::Bool(true),
                _ => MalType::Bool(false),
            },
            _ => MalType::Bool(false),
        }
    } else {
        MalType::Bool(false)
    }
}

fn addition_builtin(args: BuiltinFuncArgs) -> MalType {
    //println!("addition_builtin: {:?}", args);

    //Check to make sure we have only numeric types
    if !all_numeric(&args) {
        //println!("{:?}", args);
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
    //println!("subtraction_builtin: {:?}", args);
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

fn read_string_builtin(args: BuiltinFuncArgs) -> MalType {
    let mut result: MalType = MalType::Nil;
    for arg in args {
        if let MalType::Str(s) = arg {
            result = read_str(&s);
        }
    }
    result
}

fn slurp_builtin(args: BuiltinFuncArgs) -> MalType {
    use std::fs::File;
    use std::io::Read;

    for arg in args {
        if let MalType::Str(s) = arg {
            let mut file_res = File::open(s);
            if let Ok(mut file) = file_res {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Unable to read the file");

                return MalType::Str(contents);
            }
        }
    }

    MalType::Str("".to_string())
}

fn atom_builtin(args: BuiltinFuncArgs) -> MalType {
    for arg in args {
        return MalType::Atom(Rc::new(RefCell::new(arg)));
    }

    MalType::Atom(Rc::new(RefCell::new(MalType::Error(
        "atom takes exactly 1 argument".to_string(),
    ))))
}

fn atom_test_builtin(args: BuiltinFuncArgs) -> MalType {
    for arg in args {
        return match arg {
            MalType::Atom(_) => MalType::Bool(true),
            _ => MalType::Bool(false),
        };
    }

    MalType::Error("atom? takes exactly 1 argument".to_string())
}

fn deref_builtin(args: BuiltinFuncArgs) -> MalType {
    for arg in args {
        return match arg {
            MalType::Atom(x) => x.borrow().clone(),
            _ => MalType::Error("deref argument not an atom".to_string()),
        };
    }

    MalType::Error("deref takes exactly 1 argument".to_string())
}

fn reset_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() != 2 {
        MalType::Error("reset! takes exactly 2 arguments".to_string())
    } else {
        let atom = &args[0];
        let value = &args[1];

        match atom {
            MalType::Atom(x) => {
                x.replace(value.clone());
            }
            _ => return MalType::Error("reset! 1st argument must be an atom".to_string()),
        }

        value.clone()
    }
}

fn swap_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() < 2 {
        MalType::Error("swap! takes at least 2 arguments".to_string())
    } else {
        let atom = &args[0];
        let func = &args[1];
        let mut func_args: Vec<MalType> = Vec::new();

        match atom {
            MalType::Atom(x) => {
                func_args.push(x.borrow().clone());
            }
            _ => return MalType::Error("swap! 1st argument must be an atom".to_string()),
        }

        if args.len() > 2 {
            for arg in &args[2..] {
                func_args.push(arg.clone());
            }
        }

        match func {
            MalType::Func(f) => {
                let result = f(func_args);
                if let MalType::Atom(x) = atom {
                    x.replace(result.clone());
                }
                return result;
            }
            MalType::TCOFunc(_args, _body, _env, func) => {
                let result = func(func_args);
                if let MalType::Atom(x) = atom {
                    x.replace(result.clone());
                }
                return result;
            }
            _ => return MalType::Error("swap! 2nd argument must be a function".to_string()),
        }
    }
}

fn cons_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() < 2 {
        MalType::Error("cons takes at 2 arguments".to_string())
    } else {
        match &args[1] {
            MalType::List(l) => {
                let mut result_list: Vec<MalType> = Vec::new();
                let mut clone_list = l.clone();
                result_list.push(args[0].clone());
                result_list.append(&mut clone_list);
                MalType::List(result_list)
            }
            _ => MalType::Error("cons 2nd argument must be a list".to_string()),
        }
    }
}

fn concat_builtin(args: BuiltinFuncArgs) -> MalType {
    let mut result: Vec<MalType> = Vec::new();

    for arg in &args {
        match arg {
            MalType::List(l) => {
                result.append(&mut l.clone());
            },
            _ => return MalType::Error("concat arguments must be a list".to_string())
        }
    }

    MalType::List(result)
}
