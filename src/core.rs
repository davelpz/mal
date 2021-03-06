use eval::Environment;
use printer;
use reader::read_str;
use rep;
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
    ns.push(("nth", Rc::new(Box::new(nth_builtin))));
    ns.push(("first", Rc::new(Box::new(first_builtin))));
    ns.push(("rest", Rc::new(Box::new(rest_builtin))));

    ns
}

pub fn init_environment(env: &mut Environment) {
    for tup in create_namespace() {
        env.set(tup.0, MalType::func(tup.1, false));
    }

    env.set("*ARGV*", MalType::list(Vec::new()));

    rep("(def! not (fn* (a) (if a false true)))", env);
    rep(
        "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \")\")))))",
        env,
    );
    rep("(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw \"odd number of forms to cond\")) (cons 'cond (rest (rest xs)))))))", env);
    rep("(defmacro! or (fn* (& xs) (if (empty? xs) nil (if (= 1 (count xs)) (first xs) `(let* (or_FIXME ~(first xs)) (if or_FIXME or_FIXME (or ~@(rest xs))))))))", env);
}

fn all_numeric(args: &BuiltinFuncArgs) -> bool {
    args.iter().all(|i| i.is_int() || i.is_float())
}

fn all_int(args: &BuiltinFuncArgs) -> bool {
    args.iter().all(|i| i.is_int())
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

    MalType::nil()
}

fn println_builtin(args: BuiltinFuncArgs) -> MalType {
    println!("{}", prn_helper(args, false, " "));

    MalType::nil()
}

fn pr_str_builtin(args: BuiltinFuncArgs) -> MalType {
    MalType::string(format!("{}", prn_helper(args, true, " ")))
}

fn str_builtin(args: BuiltinFuncArgs) -> MalType {
    MalType::string(format!("{}", prn_helper(args, false, "")))
}

fn list_builtin(args: BuiltinFuncArgs) -> MalType {
    MalType::list(args)
}

fn list_test_builtin(args: BuiltinFuncArgs) -> MalType {
    match args.get(0) {
        Some(x) => MalType::bool(x.is_list()),
        _ => MalType::bool(false),
    }
}

fn empty_test_builtin(args: BuiltinFuncArgs) -> MalType {
    match args.get(0) {
        Some(x) if x.is_list() || x.is_vector() => MalType::bool(x.get_list().is_empty()),
        _ => MalType::bool(false),
    }
}

fn count_builtin(args: BuiltinFuncArgs) -> MalType {
    match args.get(0) {
        Some(x) if x.is_list() || x.is_vector() => MalType::int(x.get_list().len() as i64),
        _ => MalType::int(0),
    }
}

fn equals_builtin_helper(a: &MalType, b: &MalType) -> bool {
    //println!("a({:?}) b({:?})", a, b);
    if a.is_bool() && b.is_bool() {
        a.get_bool() == b.get_bool()
    } else if a.is_error() && b.is_error() {
        a.get_string() == b.get_string()
    } else if a.is_float() && b.is_float() {
        a.get_float() == b.get_float()
    } else if a.is_func() && b.is_func() {
        a.get_func() == b.get_func()
    } else if a.is_func_tco() && b.is_func_tco() {
        a.get_func_tco() == b.get_func_tco()
    } else if a.is_int() && b.is_int() {
        a.get_int() == b.get_int()
    } else if a.is_keyword() && b.is_keyword() {
        a.get_string() == b.get_string()
    } else if a.is_nil() && b.is_nil() {
        true
    } else if a.is_string() && b.is_string() {
        a.get_string() == b.get_string()
    } else if a.is_symbol() && b.is_symbol() {
        a.get_string() == b.get_string()
    } else if a.is_atom() && b.is_atom() {
        equals_builtin_helper(&a.get_atom(), &b.get_atom())
    } else if a.is_list() && (b.is_list() || b.is_vector()) {
        let a_list = &*a.get_list();
        let b_list = &*b.get_list();
        (a_list.len() == b_list.len()) && a_list
            .iter()
            .zip(b_list)
            .all(|(x, y)| equals_builtin_helper(x, &y))
    } else if a.is_vector() && (b.is_vector() || b.is_list()) {
        let a_list = &*a.get_list();
        let b_list = &*b.get_list();
        (a_list.len() == b_list.len()) && a_list
            .iter()
            .zip(b_list)
            .all(|(x, y)| equals_builtin_helper(x, &y))
    } else if a.is_map() && b.is_map() {
        let a_list = &*a.get_list();
        let b_list = &*b.get_list();
        (a_list.len() == b_list.len()) && a_list
            .iter()
            .zip(b_list)
            .all(|(x, y)| equals_builtin_helper(x, &y))
    } else {
        false
    }
}

fn equals_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        MalType::bool(equals_builtin_helper(&args[0], &args[1]))
    } else {
        MalType::bool(false)
    }
}

fn lt_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        if args[0].is_int() {
            if args[1].is_int() {
                return MalType::bool(args[0].get_int() < args[1].get_int());
            } else if args[1].is_float() {
                return MalType::bool(args[0].get_float() < args[1].get_float());
            }
        } else if args[0].is_float() {
            if args[1].is_int() {
                return MalType::bool(args[0].get_float() < args[1].get_float());
            } else if args[1].is_float() {
                return MalType::bool(args[0].get_float() < args[1].get_float());
            }
        }
    }

    return MalType::bool(false);
}

fn le_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        if args[0].is_int() {
            if args[1].is_int() {
                return MalType::bool(args[0].get_int() <= args[1].get_int());
            } else if args[1].is_float() {
                return MalType::bool(args[0].get_float() <= args[1].get_float());
            }
        } else if args[0].is_float() {
            if args[1].is_int() {
                return MalType::bool(args[0].get_float() <= args[1].get_float());
            } else if args[1].is_float() {
                return MalType::bool(args[0].get_float() <= args[1].get_float());
            }
        }
    }

    return MalType::bool(false);
}

fn gt_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        if args[0].is_int() {
            if args[1].is_int() {
                return MalType::bool(args[0].get_int() > args[1].get_int());
            } else if args[1].is_float() {
                return MalType::bool(args[0].get_float() > args[1].get_float());
            }
        } else if args[0].is_float() {
            if args[1].is_int() {
                return MalType::bool(args[0].get_float() > args[1].get_float());
            } else if args[1].is_float() {
                return MalType::bool(args[0].get_float() > args[1].get_float());
            }
        }
    }

    return MalType::bool(false);
}

fn ge_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() > 1 {
        if args[0].is_int() {
            if args[1].is_int() {
                return MalType::bool(args[0].get_int() >= args[1].get_int());
            } else if args[1].is_float() {
                return MalType::bool(args[0].get_float() >= args[1].get_float());
            }
        } else if args[0].is_float() {
            if args[1].is_int() {
                return MalType::bool(args[0].get_float() >= args[1].get_float());
            } else if args[1].is_float() {
                return MalType::bool(args[0].get_float() >= args[1].get_float());
            }
        }
    }

    return MalType::bool(false);
}

fn addition_builtin(args: BuiltinFuncArgs) -> MalType {
    //println!("addition_builtin: {:?}", args);

    //Check to make sure we have only numeric types
    if !all_numeric(&args) {
        //println!("{:?}", args);
        return MalType::error("Wrong types for +".to_string());
    }

    if all_int(&args) {
        let mut result: i64 = 0;
        for i in args {
            result += i.get_int()
        }
        MalType::int(result)
    } else {
        let mut result: f64 = 0.0;
        for i in args {
            result += i.get_float()
        }
        MalType::float(result)
    }
}

fn subtraction_builtin(args: BuiltinFuncArgs) -> MalType {
    //println!("subtraction_builtin: {:?}", args);
    //Check to make sure we have only numeric types
    if !all_numeric(&args) {
        return MalType::error("Wrong types for -".to_string());
    }

    if all_int(&args) {
        let mut result: i64 = args[0].get_int();
        for i in args.iter().skip(1) {
            result -= i.get_int()
        }
        MalType::int(result)
    } else {
        let mut result: f64 = args[0].get_float();
        for i in args.iter().skip(1) {
            result -= i.get_float()
        }
        MalType::float(result)
    }
}

fn multiplication_builtin(args: BuiltinFuncArgs) -> MalType {
    //Check to make sure we have only numeric types
    if !all_numeric(&args) {
        return MalType::error("Wrong types for *".to_string());
    }

    if all_int(&args) {
        let mut result: i64 = args[0].get_int();
        for i in args.iter().skip(1) {
            result *= i.get_int()
        }
        MalType::int(result)
    } else {
        let mut result: f64 = args[0].get_float();
        for i in args.iter().skip(1) {
            result *= i.get_float()
        }
        MalType::float(result)
    }
}

fn division_builtin(args: BuiltinFuncArgs) -> MalType {
    //Check to make sure we have only numeric types
    if !all_numeric(&args) {
        return MalType::error("Wrong types for /".to_string());
    }

    if all_int(&args) {
        let mut result: i64 = args[0].get_int();
        for i in args.iter().skip(1) {
            result /= i.get_int()
        }
        MalType::int(result)
    } else {
        let mut result: f64 = args[0].get_float();
        for i in args.iter().skip(1) {
            result /= i.get_float()
        }
        MalType::float(result)
    }
}

fn read_string_builtin(args: BuiltinFuncArgs) -> MalType {
    let mut result: MalType = MalType::nil();
    for arg in args {
        if arg.is_string() {
            result = read_str(&arg.get_string());
        }
    }
    result
}

fn slurp_builtin(args: BuiltinFuncArgs) -> MalType {
    use std::fs::File;
    use std::io::Read;

    for arg in args {
        if arg.is_string() {
            let mut file_res = File::open(Rc::make_mut(&mut arg.get_string()));
            if let Ok(mut file) = file_res {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Unable to read the file");

                return MalType::string(contents);
            }
        }
    }

    MalType::string("".to_string())
}

fn atom_builtin(args: BuiltinFuncArgs) -> MalType {
    for arg in args {
        return MalType::atom(arg);
    }

    MalType::atom(MalType::error("atom takes exactly 1 argument".to_string()))
}

fn atom_test_builtin(args: BuiltinFuncArgs) -> MalType {
    for arg in args {
        return MalType::bool(arg.is_atom());
    }

    MalType::error("atom? takes exactly 1 argument".to_string())
}

fn deref_builtin(args: BuiltinFuncArgs) -> MalType {
    for arg in args {
        return if arg.is_atom() {
            arg.get_atom()
        } else {
            MalType::error("deref argument not an atom".to_string())
        };
    }

    MalType::error("deref takes exactly 1 argument".to_string())
}

fn reset_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() != 2 {
        MalType::error("reset! takes exactly 2 arguments".to_string())
    } else {
        let mut temp = args[0].clone();
        let atom = &mut temp;
        let value = &args[1];

        if atom.is_atom() {
            atom.set_atom(value.clone())
        } else {
            return MalType::error("reset! 1st argument must be an atom".to_string());
        }

        value.clone()
    }
}

fn swap_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() < 2 {
        MalType::error("swap! takes at least 2 arguments".to_string())
    } else {
        let mut temp = args[0].clone();
        let atom = &mut temp;
        let func = &args[1];
        let mut func_args: Vec<MalType> = Vec::new();

        if atom.is_atom() {
            func_args.push(atom.get_atom());
        } else {
            return MalType::error("swap! 1st argument must be an atom".to_string());
        }

        if args.len() > 2 {
            for arg in &args[2..] {
                func_args.push(arg.clone());
            }
        }

        if func.is_func() {
            let (f, _is_macro) = func.get_func();
            let result = f(func_args);
            atom.set_atom(result.clone());
            return result;
        } else if func.is_func_tco() {
            let (_args, _body, _env, func, _is_macro) = func.get_func_tco();
            let result = func(func_args);
            atom.set_atom(result.clone());
            return result;
        } else {
            return MalType::error("swap! 2nd argument must be a function".to_string());
        }
    }
}

fn cons_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() < 2 {
        MalType::error("cons takes 2 arguments".to_string())
    } else {
        if args[1].is_list() || args[1].is_vector() {
            let l = &*args[1].get_list();
            let mut result_list: Vec<MalType> = Vec::new();
            let mut clone_list = l.clone();
            result_list.push(args[0].clone());
            result_list.append(&mut clone_list);
            MalType::list(result_list)
        } else {
            MalType::error("cons 2nd argument must be a list".to_string())
        }
    }
}

fn concat_builtin(args: BuiltinFuncArgs) -> MalType {
    let mut result: Vec<MalType> = Vec::new();

    for arg in &args {
        if arg.is_list() || arg.is_vector() {
            let l = &*arg.get_list();
            result.append(&mut l.clone());
        } else {
            return MalType::error("concat arguments must be a list".to_string());
        }
    }

    MalType::list(result)
}

fn nth_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() < 2 {
        MalType::error("nth takes exactly 2 arguments".to_string())
    } else {
        match args.get(0) {
            Some(x) if x.is_list() || x.is_vector() => {
                let list = x.get_list();
                match args.get(1) {
                    Some(y) if y.is_int() => {
                        let index = y.get_int();
                        if list.len() > index as usize {
                            list[index as usize].clone()
                        } else {
                            MalType::error("nth: index is greater than length of list".to_string())
                        }
                    }
                    _ => MalType::error("nth: second argument is not an int".to_string()),
                }
            }
            _ => MalType::error("nth: first argument is not a list".to_string()),
        }
    }
}

fn first_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() < 1 {
        MalType::error("first takes exactly 1 argument".to_string())
    } else {
        match args.get(0) {
            Some(x) if x.is_list() || x.is_vector() => {
                let list = x.get_list();
                if list.is_empty() {
                    MalType::nil()
                } else {
                    list[0].clone()
                }
            },
            Some(x) if x.is_nil() => {
                MalType::nil()
            },
            _ => MalType::error("first: first argument is not a list".to_string()),
        }
    }
}

fn rest_builtin(args: BuiltinFuncArgs) -> MalType {
    if args.len() < 1 {
        MalType::error("first takes exactly 1 argument".to_string())
    } else {
        match args.get(0) {
            Some(x) if x.is_list() || x.is_vector() => {
                let list = x.get_list();
                if list.is_empty() {
                    let temp: Vec<MalType> = Vec::new();
                    MalType::list(temp)
                } else {
                    if list.len() > 1 {
                        MalType::list(list[1..].to_vec())
                    } else {
                        let temp: Vec<MalType> = Vec::new();
                        MalType::list(temp)
                    }
                }
            },
            Some(x) if x.is_nil() => {
                let v: Vec<MalType> = Vec::new();
                MalType::list(v)
            },
            _ => MalType::error("first: first argument is not a list".to_string()),
        }
    }
}
