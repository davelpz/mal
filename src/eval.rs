use printer::pr_str;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use types::BuiltinFuncArgs;
use types::MalType;

pub type EnvScope = HashMap<String, MalType>;

//Defining Environment type for mal
#[derive(Debug, Clone)]
pub struct EnvironmentContents {
    pub map: EnvScope,
    pub outer: Option<Rc<RefCell<EnvironmentContents>>>,
}

impl EnvironmentContents {
    pub fn find(&self, key: String) -> Option<MalType> {
        if let Some(x) = self.map.get(&key) {
            Some(x.clone())
        } else {
            if let Some(out) = self.outer.clone() {
                out.borrow().find(key)
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub env: Rc<RefCell<EnvironmentContents>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            env: Rc::new(RefCell::new(EnvironmentContents {
                map: HashMap::new(),
                outer: None,
            })),
        }
    }

    pub fn set(&self, key: String, value: MalType) -> MalType {
        let c = value.clone();
        self.env.borrow_mut().map.insert(key, value);
        c
    }

    pub fn find(&self, key: String) -> Option<MalType> {
        let env = self.env.borrow();
        env.find(key)
    }

    pub fn get(&self, key: String) -> MalType {
        match self.find(key.clone()) {
            Some(v) => v,
            None => MalType::Error(format!("{} not found.", key)),
        }
    }

    pub fn get_inner(&self) -> Environment {
        let new_env_cont = EnvironmentContents {
            map: HashMap::new(),
            outer: Some(self.env.clone()),
        };
        Environment {
            env: Rc::new(RefCell::new(new_env_cont)),
        }
    }

    pub fn bind_exprs(&mut self, binds: &[MalType], exprs: &[MalType]) -> MalType {
        /*
        if binds.len() != exprs.len() {
            println!("binds.len() != exprs.len()");
            return MalType::Error(
                "Number of passed parameters doesn't match number of expected arguments."
                    .to_string(),
            );
        }
        */
        for (i, bind) in binds.iter().enumerate() {
            if let MalType::Symbol(b) = bind {
                //println!("bind_exprs: {:?}", b);
                if b == "&" {
                    if binds.len() > (i + 1) {
                        //println!("bind_exprs: {:?}={:?}",binds[i+1],&exprs[i..]);
                        if let MalType::Symbol(ref b2) = binds[i + 1] {
                            if exprs.len() > i {
                                //println!("{:?}", exprs.len() > i);
                                //println!("{:?}", exprs.len());
                                //println!("{:?}", i);
                                //println!("{:?}", exprs[i..].to_vec());
                                self.set(b2.clone(), MalType::List(exprs[i..].to_vec()));
                            } else {
                                self.set(b2.clone(), MalType::List(Vec::new()));
                            }
                        }
                        break;
                    }
                } else {
                    //println!("bind_exprs: setting");
                    if exprs.len() > i {
                        self.set(b.clone(), exprs[i].clone());
                    }
                }
            } else {
                return MalType::Error("Non Symbol in parameter list".to_string());
            }
        }
        MalType::Nil
    }
}

fn new_let_env(bind_list: &MalType, env: &mut Environment) -> Option<Environment> {
    let mut new_env = env.get_inner();
    match bind_list {
        MalType::List(l) | MalType::Vector(l) => {
            if l.len() % 2 == 1 {
                return None;
            }
            for chunk in l.chunks(2) {
                if !chunk.is_empty() {
                    match &chunk[0] {
                        MalType::Symbol(sym) => {
                            let three = eval(&chunk[1], &mut new_env);
                            match three {
                                MalType::Error(_) => {
                                    return None;
                                }
                                _ => {
                                    new_env.set(sym.to_string(), three);
                                }
                            }
                        }
                        _ => return None,
                    }
                }
            }
        }
        _ => return None,
    }

    Some(new_env)
}

fn eval_list(t: &MalType, env: &mut Environment) -> MalType {
    let mut eval_list_ast = eval_ast(t, env);
    if let MalType::List(ref mut eval_list) = eval_list_ast {
        let mut first = &eval_list[0];
        if let MalType::Error(_) = first {
            first.clone()
        } else if let MalType::Func(f) = first {
            f(eval_list[1..].to_vec())
        } else {
            MalType::Error(format!("{} not found.", pr_str(first, true)))
        }
    } else {
        MalType::Error("internal error: eval_ast of List did not return a List".to_string())
    }
}

pub fn eval(mut t: &MalType, env: &mut Environment) -> MalType {
    let mut new_env : Environment = env.clone();

    loop {
        match t {
            MalType::Error(_) => return t.clone(),
            MalType::List(list) if list.is_empty() => return t.clone(),
            MalType::List(uneval_list) if !uneval_list.is_empty() => {
                let first = &uneval_list[0];
                if let MalType::Error(_) = first {
                    //don't think this is needed
                    return MalType::Error(pr_str(first, true).to_string())
                } else if let MalType::Symbol(s) = first {
                    if s == "def!" {
                        let second = &uneval_list[1];
                        let third = eval(&uneval_list[2], env);
                        match third {
                            MalType::Error(_) => return third,
                            _ => return env.set(second.get_symbol_string(), third),
                        }
                    } else if s == "let*" {
                        new_env = new_let_env(&uneval_list[1], &mut new_env).unwrap();
                        //env = &mut new_env;
                        t = &uneval_list[2];
                    } else if s == "do" {
                        if let MalType::List(l) =
                            eval_ast(&MalType::List(uneval_list[1..].to_vec()), env)
                        {
                            return l.last().unwrap().clone()
                        } else {
                            return MalType::Error(
                                "Internal Error: eval_ast of list did not return a list"
                                    .to_string(),
                            )
                        }
                    } else if s == "if" {
                        match eval(&uneval_list[1], env) {
                            MalType::Error(x) => return MalType::Error(x),
                            MalType::Nil | MalType::Bool(false) => {
                                if uneval_list.len() > 3 {
                                    return eval(&uneval_list[3], env)
                                } else {
                                    return MalType::Nil
                                }
                            }
                            _ => {
                                if uneval_list.len() > 2 {
                                    return eval(&uneval_list[2], env)
                                } else {
                                    return MalType::Nil
                                }
                            }
                        }
                    } else if s == "fn*" {
                        match &uneval_list[1] {
                            MalType::List(binds) | MalType::Vector(binds) => {
                                //need to clone everything to prevent dangaling references
                                let binds_clone = binds.clone();
                                let function_body = uneval_list[2].clone();

                                //create new clone environment, to cut ties to passed in env
                                let new_env = env.clone();

                                //println!("{:?}", function_body);
                                let new_func = move |args: BuiltinFuncArgs| {
                                    //println!("do_fn_special_atom: {:?}", args);
                                    //clone again to gut ties to outer function body
                                    let mut new_func_env = new_env.get_inner();

                                    //bind function arguments
                                    new_func_env.bind_exprs(&binds_clone, &args);

                                    //finally call the function
                                    eval(&function_body, &mut new_func_env)
                                };
                                return MalType::Func(Rc::new(Box::new(new_func)))
                            }
                            _ => return MalType::Error(format!(
                                "bind list is not a list: {} ",
                                pr_str(&uneval_list[1], true)
                            )),
                        }

                    //do_fn_special_atom(uneval_list, env)
                    } else {
                        return eval_list(t, env)
                    }
                } else {
                    return eval_list(t, env)
                    //MalType::Error(format!("{} not found.", pr_str(first)))
                }
            }
            _ => return eval_ast(t, env),
        }
    }
}

pub fn eval_ast(t: &MalType, env: &mut Environment) -> MalType {
    match t {
        MalType::Symbol(s) => {
            let lookup = env.get(s.to_string());
            match lookup {
                //MalType::Func(f) => MalType::Func(f.clone()),
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
    use core::init_environment;
    use reader::read_str;

    #[test]
    fn evironmental_test() {
        let mut env = Environment::new();
        init_environment(&mut env);

        env.set("key1".to_string(), MalType::Int(1));
        env.set("key2".to_string(), MalType::Int(2));
        env.set("key3".to_string(), MalType::Int(3));

        assert_eq!(env.get("key1".to_string()), MalType::Int(1));
        assert_eq!(env.get("key2".to_string()), MalType::Int(2));
        assert_eq!(env.get("key3".to_string()), MalType::Int(3));
        assert_eq!(
            env.get("won't find".to_string()),
            MalType::Error("won\'t find not found.".to_string())
        );

        let inner = env.get_inner();
        assert_eq!(inner.get("key1".to_string()), MalType::Int(1));
        assert_eq!(
            inner.get("won't find".to_string()),
            MalType::Error("won\'t find not found.".to_string())
        );

        inner.set("key3".to_string(), MalType::Int(33));
        assert_eq!(inner.get("key3".to_string()), MalType::Int(33));

        let mut inner2 = inner.get_inner();
        assert_eq!(inner2.get("key1".to_string()), MalType::Int(1));
        assert_eq!(
            inner2.get("won't find".to_string()),
            MalType::Error("won\'t find not found.".to_string())
        );

        inner2.set("key3".to_string(), MalType::Int(333));
        assert_eq!(inner2.get("key3".to_string()), MalType::Int(333));

        let mut bind: Vec<MalType> = Vec::new();
        let mut expr: Vec<MalType> = Vec::new();

        bind.push(MalType::Symbol("a".to_string()));
        bind.push(MalType::Symbol("b".to_string()));
        bind.push(MalType::Symbol("c".to_string()));

        expr.push(MalType::Int(666));
        expr.push(MalType::Int(777));
        expr.push(MalType::Int(888));

        inner2.bind_exprs(&bind, &expr);
        assert_eq!(inner2.get("a".to_string()), MalType::Int(666));
        assert_eq!(inner2.get("b".to_string()), MalType::Int(777));
        assert_eq!(inner2.get("c".to_string()), MalType::Int(888));

        env.set("newSymbol".to_string(), MalType::Int(456));
        assert_eq!(inner2.get("newSymbol".to_string()), MalType::Int(456));
    }

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

        let result_vec: Vec<MalType> = vec![MalType::Str("a".to_string()), MalType::Int(15)];
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
            println!("{:?}", tup.0);
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }

    #[test]
    fn eval_test_step4() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let mut tests: Vec<(&str, MalType)> = Vec::new();

        //;; Testing list functions
        tests.push(("(list)", MalType::List(Vec::new())));
        tests.push(("(list? (list))", MalType::Bool(true)));
        tests.push(("(empty? (list))", MalType::Bool(true)));
        tests.push(("(empty? (list 1))", MalType::Bool(false)));
        let mut v1 = Vec::new();
        v1.push(MalType::Int(1));
        v1.push(MalType::Int(2));
        v1.push(MalType::Int(3));
        tests.push(("(list 1 2 3)", MalType::List(v1)));
        tests.push(("(count (list 1 2 3))", MalType::Int(3)));
        tests.push(("(count (list))", MalType::Int(0)));
        tests.push(("(count nil)", MalType::Int(0)));
        tests.push((
            "(if (> (count (list 1 2 3)) 3) \"yes\" \"no\")",
            MalType::Str("no".to_string()),
        ));
        tests.push((
            "(if (>= (count (list 1 2 3)) 3) \"yes\" \"no\")",
            MalType::Str("yes".to_string()),
        ));

        //;; Testing if form
        tests.push(("(if true 7 8)", MalType::Int(7)));
        tests.push(("(if false 7 8)", MalType::Int(8)));
        tests.push(("(if true (+ 1 7) (+ 1 8))", MalType::Int(8)));
        tests.push(("(if false (+ 1 7) (+ 1 8))", MalType::Int(9)));
        tests.push(("(if nil 7 8)", MalType::Int(8)));
        tests.push(("(if 0 7 8)", MalType::Int(7)));
        tests.push(("(if \"\" 7 8)", MalType::Int(7)));
        tests.push(("(if (list) 7 8)", MalType::Int(7)));
        tests.push(("(if (list 1 2 3) 7 8)", MalType::Int(7)));
        tests.push(("(= (list) nil)", MalType::Bool(false)));

        //;; Testing 1-way if form
        tests.push(("(if false (+ 1 7))", MalType::Nil));
        tests.push(("(if nil 8 7)", MalType::Int(7)));
        tests.push(("(if true (+ 1 7))", MalType::Int(8)));

        //;; Testing basic conditionals
        tests.push(("(= 2 1)", MalType::Bool(false)));
        tests.push(("(= 1 1)", MalType::Bool(true)));
        tests.push(("(= 1 2)", MalType::Bool(false)));
        tests.push(("(= 1 (+ 1 1))", MalType::Bool(false)));
        tests.push(("(= 2 (+ 1 1))", MalType::Bool(true)));
        tests.push(("(= nil 1)", MalType::Bool(false)));
        tests.push(("(= nil nil)", MalType::Bool(true)));
        tests.push(("(> 2 1)", MalType::Bool(true)));
        tests.push(("(> 1 1)", MalType::Bool(false)));
        tests.push(("(> 1 2)", MalType::Bool(false)));
        tests.push(("(>= 2 1)", MalType::Bool(true)));
        tests.push(("(>= 1 1)", MalType::Bool(true)));
        tests.push(("(>= 1 2)", MalType::Bool(false)));
        tests.push(("(< 2 1)", MalType::Bool(false)));
        tests.push(("(< 1 1)", MalType::Bool(false)));
        tests.push(("(< 1 2)", MalType::Bool(true)));
        tests.push(("(<= 2 1)", MalType::Bool(false)));
        tests.push(("(<= 1 1)", MalType::Bool(true)));
        tests.push(("(<= 1 2)", MalType::Bool(true)));

        //;; Testing equality
        tests.push(("(= 1 1)", MalType::Bool(true)));
        tests.push(("(= 0 0)", MalType::Bool(true)));
        tests.push(("(= 1 0)", MalType::Bool(false)));
        tests.push(("(= \"\" \"\")", MalType::Bool(true)));
        tests.push(("(= \"abc\" \"abc\")", MalType::Bool(true)));
        tests.push(("(= \"abc\" \"\")", MalType::Bool(false)));
        tests.push(("(= \"\" \"abc\")", MalType::Bool(false)));
        tests.push(("(= \"abc\" \"def\")", MalType::Bool(false)));
        tests.push(("(= \"abc\" \"ABC\")", MalType::Bool(false)));
        tests.push(("(= true true)", MalType::Bool(true)));
        tests.push(("(= false false)", MalType::Bool(true)));
        tests.push(("(= nil nil)", MalType::Bool(true)));
        tests.push(("(= (list) (list))", MalType::Bool(true)));
        tests.push(("(= (list 1 2) (list 1 2))", MalType::Bool(true)));
        tests.push(("(= (list 1) (list))", MalType::Bool(false)));
        tests.push(("(= (list) (list 1))", MalType::Bool(false)));
        tests.push(("(= 0 (list))", MalType::Bool(false)));
        tests.push(("(= (list) 0)", MalType::Bool(false)));
        tests.push(("(= (list) \"\")", MalType::Bool(false)));
        tests.push(("(= \"\" (list))", MalType::Bool(false)));

        //;; Testing builtin and user defined functions
        tests.push(("(+ 1 2)", MalType::Int(3)));
        tests.push(("( (fn* (a b) (+ b a)) 3 4)", MalType::Int(7)));
        tests.push(("( (fn* () 4) )", MalType::Int(4)));
        tests.push(("( (fn* (f x) (f x)) (fn* (a) (+ 1 a)) 7)", MalType::Int(8)));

        //;; Testing closures
        tests.push(("( ( (fn* (a) (fn* (b) (+ a b))) 5) 7)", MalType::Int(12)));

        eval(
            &read_str("(def! gen-plus5 (fn* () (fn* (b) (+ 5 b))))"),
            &mut env,
        );
        eval(&read_str("(def! plus5 (gen-plus5))"), &mut env);
        tests.push(("(plus5 7)", MalType::Int(12)));

        eval(
            &read_str("(def! gen-plusX (fn* (x) (fn* (b) (+ x b))))"),
            &mut env,
        );
        eval(&read_str("(def! plus7 (gen-plusX 7))"), &mut env);
        tests.push(("(plus7 8)", MalType::Int(15)));

        //;; Testing do form
        tests.push(("(do (prn \"prn output1\"))", MalType::Nil));
        tests.push(("(do (prn \"prn output2\") 7)", MalType::Int(7)));
        tests.push((
            "(do (prn \"prn output1\") (prn \"prn output2\") (+ 1 2))",
            MalType::Int(3),
        ));
        tests.push(("(do (def! a 6) 7 (+ a 8))", MalType::Int(14)));
        tests.push(("a", MalType::Int(6)));

        //;; Testing special form case-sensitivity
        eval(&read_str("(def! DO (fn* (a) 7))"), &mut env);
        tests.push(("(DO 3)", MalType::Int(7)));

        //;; Testing recursive sumdown function
        eval(
            &read_str("(def! sumdown (fn* (N) (if (> N 0) (+ N (sumdown  (- N 1))) 0)))"),
            &mut env,
        );
        tests.push(("(sumdown 1)", MalType::Int(1)));
        tests.push(("(sumdown 2)", MalType::Int(3)));
        tests.push(("(sumdown 6)", MalType::Int(21)));

        //;; Testing recursive fibonacci function
        eval(
            &read_str("(def! fib (fn* (N) (if (= N 0) 1 (if (= N 1) 1 (+ (fib (- N 1)) (fib (- N 2)))))))"),
            &mut env,
        );
        tests.push(("(fib 1)", MalType::Int(1)));
        tests.push(("(fib 2)", MalType::Int(2)));
        tests.push(("(fib 4)", MalType::Int(5)));
        tests.push(("(fib 10)", MalType::Int(89)));

        //;; Testing language defined not function
        tests.push(("(not false)", MalType::Bool(true)));
        tests.push(("(not nil)", MalType::Bool(true)));
        tests.push(("(not true)", MalType::Bool(false)));
        tests.push(("(not \"a\")", MalType::Bool(false)));
        tests.push(("(not 0)", MalType::Bool(false)));

        //;; Testing string quoting
        tests.push(("\"\"", MalType::Str("".to_string())));
        tests.push(("\"abc\"", MalType::Str("abc".to_string())));
        tests.push(("\"abc  def\"", MalType::Str("abc  def".to_string())));
        tests.push(("\"\\\"\"", MalType::Str("\"".to_string())));
        tests.push((
            "\"abc\ndef\nghi\"",
            MalType::Str("abc\ndef\nghi".to_string()),
        ));
        tests.push((
            "\"abc\\\\def\\\\ghi\"",
            MalType::Str("abc\\def\\ghi".to_string()),
        ));
        tests.push(("\"\\\\n\"", MalType::Str("\\n".to_string())));

        //;; Testing pr-str
        tests.push(("(pr-str)", MalType::Str("".to_string())));
        tests.push(("(pr-str \"\")", MalType::Str("".to_string())));
        tests.push(("(pr-str \"abc\")", MalType::Str("\"abc\"".to_string())));
        tests.push((
            "(pr-str \"abc def\" \"ghi jkl\")",
            MalType::Str("\"abc def\" \"ghi jkl\"".to_string()),
        ));
        tests.push(("(pr-str \"\\\"\")", MalType::Str("\"\\\"\"".to_string())));
        tests.push((
            "(pr-str (list 1 2 \"abc\" \"\\\"\") \"def\")",
            MalType::Str("(1 2 \"abc\" \"\\\"\") \"def\"".to_string()),
        ));
        tests.push((
            "(pr-str \"abc\\ndef\\nghi\")",
            MalType::Str("\"abc\\ndef\\nghi\"".to_string()),
        ));
        tests.push((
            "(pr-str \"abc\\\\def\\\\ghi\")",
            MalType::Str("\"abc\\\\def\\\\ghi\"".to_string()),
        ));
        tests.push(("(pr-str (list))", MalType::Str("()".to_string())));

        //;; Testing str
        tests.push(("(str)", MalType::Str("".to_string())));
        tests.push(("(str \"\")", MalType::Str("".to_string())));
        tests.push(("(str \"abc\")", MalType::Str("abc".to_string())));
        tests.push(("(str \"\\\"\")", MalType::Str("\"".to_string())));
        tests.push(("(str 1 \"abc\" 3)", MalType::Str("1abc3".to_string())));
        tests.push((
            "(str \"abc  def\" \"ghi jkl\")",
            MalType::Str("abc  defghi jkl".to_string()),
        ));
        tests.push((
            "(str \"abc\\\\def\\\\ghi\")",
            MalType::Str("abc\\def\\ghi".to_string()),
        ));
        tests.push((
            "(str (list 1 2 \"abc\" \"\\\"\") \"def\")",
            MalType::Str("(1 2 abc \")def".to_string()),
        ));
        tests.push(("(str (list))", MalType::Str("()".to_string())));

        //;; Testing prn
        tests.push(("(prn)", MalType::Nil));
        tests.push(("(prn \"\")", MalType::Nil));
        tests.push(("(prn \"abc\")", MalType::Nil));
        tests.push(("(prn \"abc  def\" \"ghi jkl\")", MalType::Nil));
        tests.push(("(prn \"\\\"\")", MalType::Nil));
        tests.push(("(prn \"abc\ndef\nghi\")", MalType::Nil));
        tests.push(("(prn \"abc\\def\\ghi\")", MalType::Nil));
        tests.push(("(prn (list 1 2 \"abc\" \"\\\"\") \"def\")", MalType::Nil));

        //;; Testing println
        tests.push(("(println)", MalType::Nil));
        tests.push(("(println \"\")", MalType::Nil));
        tests.push(("(println \"abc\")", MalType::Nil));
        tests.push(("(println \"abc  def\" \"ghi jkl\")", MalType::Nil));
        tests.push(("(println \"\\\"\")", MalType::Nil));
        tests.push(("(println \"abc\ndef\nghi\")", MalType::Nil));
        tests.push(("(println \"abc\\def\\ghi\")", MalType::Nil));
        tests.push((
            "(println (list 1 2 \"abc\" \"\\\"\") \"def\")",
            MalType::Nil,
        ));

        //;; Testing keywords
        tests.push(("(= :abc :abc)", MalType::Bool(true)));
        tests.push(("(= :abc :def)", MalType::Bool(false)));
        tests.push(("(= :abc \":abc\")", MalType::Bool(false)));

        //;; Testing vector truthiness
        tests.push(("(if [] 7 8)", MalType::Int(7)));

        //;; Testing vector printing
        tests.push((
            "(pr-str [1 2 \"abc\" \"\\\"\"] \"def\")",
            MalType::Str("[1 2 \"abc\" \"\\\"\"] \"def\"".to_string()),
        ));
        tests.push(("(pr-str [])", MalType::Str("[]".to_string())));
        tests.push((
            "(str [1 2 \"abc\" \"\\\"\"] \"def\")",
            MalType::Str("[1 2 abc \"]def".to_string()),
        ));
        tests.push(("(str [])", MalType::Str("[]".to_string())));

        //;; Testing vector functions
        tests.push(("(count [1 2 3])", MalType::Int(3)));
        tests.push(("(empty? [1 2 3])", MalType::Bool(false)));
        tests.push(("(empty? [])", MalType::Bool(true)));
        tests.push(("(list? [4 5 6])", MalType::Bool(false)));

        //;; Testing vector equality
        tests.push(("(= [] (list))", MalType::Bool(true)));
        tests.push(("(= [7 8] [7 8])", MalType::Bool(true)));
        tests.push(("(= (list 1 2) [1 2])", MalType::Bool(true)));
        tests.push(("(= (list 1) [])", MalType::Bool(false)));
        tests.push(("(= [] [1])", MalType::Bool(false)));
        tests.push(("(= 0 [])", MalType::Bool(false)));
        tests.push(("(= [] 0)", MalType::Bool(false)));
        tests.push(("(= [] \"\")", MalType::Bool(false)));
        tests.push(("(= \"\" [])", MalType::Bool(false)));

        //;; Testing vector parameter lists
        tests.push(("( (fn* [] 4) )", MalType::Int(4)));
        tests.push(("( (fn* [f x] (f x)) (fn* [a] (+ 1 a)) 7)", MalType::Int(8)));

        //;; Nested vector/list equality
        tests.push(("(= [(list)] (list []))", MalType::Bool(true)));
        tests.push((
            "(= [1 2 (list 3 4 [5 6])] (list 1 2 [3 4 (list 5 6)]))",
            MalType::Bool(true),
        ));

        for tup in tests {
            //println!("{:?}",tup.0);
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }

    #[test]
    fn eval_test_step4_deferred() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let mut tests: Vec<(&str, MalType)> = Vec::new();

        //;; Testing variable length arguments
        tests.push(("( (fn* (& more) (count more)) 1 2 3)", MalType::Int(3)));
        tests.push(("( (fn* (& more) (list? more)) 1 2 3)", MalType::Bool(true)));
        tests.push(("( (fn* (& more) (count more)) 1)", MalType::Int(1)));
        tests.push(("( (fn* (& more) (count more)) )", MalType::Int(0)));
        tests.push(("( (fn* (& more) (list? more)) )", MalType::Bool(true)));
        tests.push(("( (fn* (a & more) (list? more)) )", MalType::Bool(true)));
        tests.push(("( (fn* (a & more) (count more)) 1 2 3)", MalType::Int(2)));
        tests.push(("( (fn* (a & more) (count more)) 1)", MalType::Int(0)));
        tests.push(("( (fn* (a & more) (list? more)) 1)", MalType::Bool(true)));

        for tup in tests {
            //println!("{:?}",tup.0);
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }
}
