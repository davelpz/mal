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

pub fn eval(t: &MalType, env: &mut Environment) -> MalType {
    match t {
        MalType::Error(_) => t.clone(),
        MalType::List(list) if list.is_empty() => t.clone(),
        MalType::List(uneval_list) if !uneval_list.is_empty() => {
            let first = &uneval_list[0];
            if let MalType::Error(_) = first {
                //don't think this is needed
                MalType::Error(pr_str(first).to_string())
            } else if let MalType::Symbol(s) = first {
                if s == "def!" {
                    let second = &uneval_list[1];
                    let third = eval(&uneval_list[2], env);
                    //println!("{:?}",second);
                    //println!("{:?}",third);
                    match third {
                        MalType::Error(_) => third,
                        _ => env.set(second.get_symbol_string(), third),
                    }
                } else if s == "let*" {
                    let mut new_env = env.get_inner();
                    let second = &uneval_list[1];

                    match second {
                        MalType::List(l) | MalType::Vector(l) => {
                            if l.len() % 2 == 1 {
                                return MalType::Error(
                                    "Error: let*, can't set, odd number in set list.".to_string(),
                                );
                            }
                            for chunk in l.chunks(2) {
                                if !chunk.is_empty() {
                                    match &chunk[0] {
                                        MalType::Symbol(sym) => {
                                            let three = eval(&chunk[1], &mut new_env);
                                            match three {
                                                MalType::Error(_) => {
                                                    return three;
                                                }
                                                _ => {
                                                    new_env.set(sym.to_string(), three);
                                                }
                                            }
                                        }
                                        _ => {
                                            return MalType::Error(
                                                "Error: let*, can't set, not symbol.".to_string(),
                                            )
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            return MalType::Error(
                                "Error: let*, second argument is not a list.".to_string(),
                            )
                        }
                    }

                    eval(&uneval_list[2], &mut new_env)
                } else {
                    let eval_list_ast = eval_ast(t, env);
                    if let MalType::List(eval_list) = eval_list_ast {
                        let first = &eval_list[0];
                        if let MalType::Error(_) = first {
                            first.clone()
                        } else if let MalType::Func(f) = first {
                            f(eval_list[1..].to_vec())
                        } else {
                            MalType::Error(format!("{} not found.", pr_str(first)))
                        }
                    } else {
                        MalType::Error(
                            "internal error: eval_ast of List did not return a List".to_string(),
                        )
                    }
                }
            } else {
                MalType::Error(format!("{} not found.", pr_str(first)))
            }
        }
        _ => eval_ast(t, env),
    }
}

pub fn eval_ast(t: &MalType, env: &mut Environment) -> MalType {
    match t {
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
    fn eval_test_step2() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let mut tests: Vec<(&str, MalType)> = Vec::new();
        tests.push(("(+ 1 2)", MalType::Int(3)));
        tests.push(("(+ 5 (* 2 3))", MalType::Int(11)));
        tests.push(("(- (+ 5 (* 2 3)) 3)", MalType::Int(8)));
        tests.push(("(/ (- (+ 5 (* 2 3)) 3) 4)", MalType::Int(2)));
        tests.push(("(/ (- (+ 515 (* 87 311)) 302) 27)", MalType::Int(1010)));
        tests.push(("(* -3 6)", MalType::Int(-18)));
        tests.push(("(/ (- (+ 515 (* -87 311)) 296) 27)", MalType::Int(-994)));
        tests.push(("(abc 1 2 3)", MalType::Error("abc not found.".to_string())));

        let empty_vec: Vec<MalType> = vec![];
        tests.push(("()", MalType::List(empty_vec)));

        let result_vec: Vec<MalType> = vec![MalType::Int(1), MalType::Int(2), MalType::Int(3)];
        tests.push(("[1 2 (+ 1 2)]", MalType::Vector(result_vec)));

        let result_vec: Vec<MalType> = vec![MalType::Str("\"a\"".to_string()), MalType::Int(15)];
        tests.push(("{\"a\" (+ 7 8)}", MalType::Map(result_vec)));

        let result_vec: Vec<MalType> = vec![MalType::KeyWord(":a".to_string()), MalType::Int(15)];
        tests.push(("{:a (+ 7 8)}", MalType::Map(result_vec)));

        for tup in tests {
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }

    #[test]
    fn eval_test_step3() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let mut tests: Vec<(&str, MalType)> = Vec::new();
        tests.push(("(+ 1 2)", MalType::Int(3)));
        tests.push(("(/ (- (+ 5 (* 2 3)) 3) 4)", MalType::Int(2)));
        tests.push(("(def! x 3)", MalType::Int(3)));
        tests.push(("x", MalType::Int(3)));
        tests.push(("(def! x 4)", MalType::Int(4)));
        tests.push(("x", MalType::Int(4)));
        tests.push(("(def! y (+ 1 7))", MalType::Int(8)));
        tests.push(("y", MalType::Int(8)));
        tests.push(("(def! mynum 111)", MalType::Int(111)));
        tests.push(("(def! MYNUM 222)", MalType::Int(222)));
        tests.push(("mynum", MalType::Int(111)));
        tests.push(("MYNUM", MalType::Int(222)));
        tests.push(("(abc 1 2 3)", MalType::Error("abc not found.".to_string())));
        tests.push(("(def! w 123)", MalType::Int(123)));
        tests.push((
            "(def! w (abc))",
            MalType::Error("abc not found.".to_string()),
        ));
        tests.push(("(def! w 123)", MalType::Int(123)));
        tests.push(("(let* (x 9) x)", MalType::Int(9)));
        tests.push(("(let* (z 9) z)", MalType::Int(9)));
        tests.push(("x", MalType::Int(4)));
        tests.push(("(let* (z (+ 2 3)) (+ 1 z))", MalType::Int(6)));
        tests.push(("(let* (p (+ 2 3) q (+ 2 p)) (+ p q))", MalType::Int(12)));
        tests.push(("(def! y (let* (z 7) z))", MalType::Int(7)));
        tests.push(("y", MalType::Int(7)));
        tests.push(("(def! a 4)", MalType::Int(4)));
        tests.push(("(let* (q 9) q)", MalType::Int(9)));
        tests.push(("(let* (q 9) a)", MalType::Int(4)));
        tests.push(("(let* (z 2) (let* (q 9) a))", MalType::Int(4)));
        tests.push(("(let* (x 4) (def! a 5))", MalType::Int(5)));
        tests.push(("a", MalType::Int(4)));
        tests.push(("(let* [z 9] z)", MalType::Int(9)));
        tests.push(("(let* [p (+ 2 3) q (+ 2 p)] (+ p q))", MalType::Int(12)));

        let mut v1 = Vec::new();
        v1.push(MalType::Int(3));
        v1.push(MalType::Int(4));
        v1.push(MalType::Int(5));
        let mut v2 = Vec::new();
        v2.push(MalType::Int(6));
        v2.push(MalType::Int(7));
        v1.push(MalType::Vector(v2));
        v1.push(MalType::Int(8));
        tests.push(("(let* (a 5 b 6) [3 4 a [b 7] 8])", MalType::Vector(v1)));

        for tup in tests {
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }
}
