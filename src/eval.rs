use printer::pr_str;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use types::BuiltinFuncArgs;
use types::MalType;

pub type EnvScope = HashMap<String, MalType>;

//Defining Environment type for mal

#[derive(Debug, Clone)]
pub struct Environment {
    pub map: Rc<RefCell<EnvScope>>,
    pub outer: Option<Rc<RefCell<Environment>>>,
}

impl PartialEq for Environment {
    fn eq(&self, _other: &Environment) -> bool {
        false
    }
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            map: Rc::new(RefCell::new(HashMap::new())),
            outer: None,
        }
    }

    pub fn set(&self, key: &str, value: MalType) -> MalType {
        let c = value.clone();
        self.map.borrow_mut().insert(key.to_string(), value);
        c
    }

    pub fn find(&self, key: &str) -> Option<MalType> {
        if let Some(x) = self.map.borrow().get(key) {
            Some(x.clone())
        } else {
            if let Some(out) = self.outer.clone() {
                out.borrow().find(key)
            } else {
                None
            }
        }
    }

    pub fn get(&self, key: &str) -> MalType {
        match self.find(key.clone()) {
            Some(v) => v,
            None => MalType::error(format!("{} not found.", key)),
        }
    }

    pub fn get_inner(&self) -> Environment {
        Environment {
            map: Rc::new(RefCell::new(HashMap::new())),
            outer: Some(Rc::new(RefCell::new(self.clone()))),
        }
    }

    pub fn get_root(&self) -> Environment {
        match self.outer {
            None => self.clone(),
            Some(ref e) => e.borrow().get_root(),
        }
    }

    pub fn bind_exprs(&mut self, binds: &[MalType], exprs: &[MalType]) -> MalType {
        for (i, bind) in binds.iter().enumerate() {
            if bind.is_symbol() {
                let b = bind.get_string();
                //println!("bind_exprs: {:?}", b);
                if *b == "&" {
                    if binds.len() > (i + 1) {
                        //println!("bind_exprs: {:?}={:?}",binds[i+1],&exprs[i..]);
                        if binds[i + 1].is_symbol() {
                            let b2 = binds[i + 1].get_string();
                            if exprs.len() > i {
                                //println!("{:?}", exprs.len() > i);
                                //println!("{:?}", exprs.len());
                                //println!("{:?}", i);
                                //println!("{:?}", exprs[i..].to_vec());
                                self.set(&b2, MalType::list(exprs[i..].to_vec()));
                            } else {
                                self.set(&b2, MalType::list(Vec::new()));
                            }
                        }
                        break;
                    }
                } else {
                    //println!("bind_exprs: setting");
                    if exprs.len() > i {
                        self.set(&b, exprs[i].clone());
                    }
                }
            } else {
                return MalType::error("Non Symbol in parameter list".to_string());
            }
        }
        MalType::nil()
    }
}

fn new_let_env(bind_list: &MalType, env: &mut Environment) -> Option<Environment> {
    let mut new_env = env.get_inner();
    if bind_list.is_list() || bind_list.is_vector() {
        let l = bind_list.get_list();
        if l.len() % 2 == 1 {
            return None;
        }
        for chunk in l.chunks(2) {
            if !chunk.is_empty() {
                if chunk[0].is_symbol() {
                    let sym = chunk[0].get_string();
                    let three = eval(&chunk[1], &mut new_env);
                    if three.is_error() {
                        return None;
                    } else {
                        new_env.set(&sym, three);
                    }
                } else {
                    return None;
                }
            }
        }
    } else {
        return None;
    }

    Some(new_env)
}

pub fn is_pair(ast: &MalType) -> bool {
    if ast.is_list() || ast.is_vector() {
        !ast.get_list().is_empty()
    } else {
        false
    }
}

pub fn quasiquote(ast: &MalType) -> MalType {
    if is_pair(ast) {
        let l = ast.get_list();
        if l.is_empty() {
            return MalType::nil();
        }

        if l[0].is_symbol() {
            let sym = l[0].get_string();
            if l.len() == 1 {
                return MalType::nil();
            }

            if *sym == "unquote" {
                return l[1].clone();
            }
        }

        if is_pair(&l[0]) {
            let l2 = l[0].get_list();
            if l2.is_empty() {
                return MalType::nil();
            }

            if l2[0].is_symbol() {
                let sym2 = l2[0].get_string();
                if l2.len() == 1 {
                    return MalType::nil();
                }

                if *sym2 == "splice-unquote" {
                    let mut list: Vec<MalType> = Vec::new();
                    list.push(MalType::symbol("concat".to_string()));
                    list.push(l2[1].clone());
                    list.push(quasiquote(&MalType::list(l[1..].to_vec())));
                    return MalType::list(list);
                }
            }
        }

        let mut list: Vec<MalType> = Vec::new();
        list.push(MalType::symbol("cons".to_string()));
        list.push(quasiquote(&l[0]));
        list.push(quasiquote(&MalType::list(l[1..].to_vec())));
        return MalType::list(list);
    } else {
        let mut list: Vec<MalType> = Vec::new();
        list.push(MalType::symbol("quote".to_string()));
        list.push(ast.clone());
        return MalType::list(list);
    }
}

fn is_macro_call(ast: &MalType, env: &mut Environment) -> bool {
    if ast.is_list() {
        let l = ast.get_list();
        if !l.is_empty() {
            if l[0].is_symbol() {
                let sym = l[0].get_string();
                let val = env.get(&sym);
                return val.is_macro();
            }
        }
    }
    false
}

fn macroexpand(ast_incomming: &MalType, env: &mut Environment) -> MalType {
    let mut ast = ast_incomming.clone();
    let mut is_macro = is_macro_call(&ast, env);

    while is_macro {
        let l = ast.get_list();
        if l[0].is_symbol() {
            let sym = l[0].get_string();
            let val = env.get(&sym);
            if val.is_func() {
                let (f, _) = val.get_func();
                ast = f(l[1..].to_vec());
            } else if val.is_func_tco() {
                let (_, _, _, f, _) = val.get_func_tco();
                ast = f(l[1..].to_vec());
            }
            is_macro = is_macro_call(&ast, env);
        }
    }

    ast
}

pub fn eval(t1: &MalType, env: &mut Environment) -> MalType {
    let mut ast = t1.clone();
    let mut eval_env: Environment = env.clone();

    //println!("eval {:?}", ast);

    loop {
        if !ast.is_list() {
            return eval_ast(&ast, &mut eval_env);
        }

        ast = macroexpand(&ast, env);

        if ast.is_error() {
            return ast;
        } else if ast.is_list() {
            let uneval_list = ast.get_list();
            if uneval_list.is_empty() {
                return ast;
            }

            let first = &uneval_list[0];
            if first.is_error() {
                //don't think this is needed
                return MalType::error(pr_str(first, true).to_string());
            } else if first.is_symbol() {
                let s = first.get_string();
                if *s == "eval" {
                    let second = eval(&uneval_list[1], &mut eval_env);
                    let mut root_env = eval_env.get_root();

                    //println!("in eval after eval_ast: {:?}", second);
                    let eval_result = eval(&second, &mut root_env);
                    //println!("in eval after eval: {:?}", eval_result);
                    return eval_result;
                } else if *s == "def!" {
                    let second = &uneval_list[1];
                    let third = eval(&uneval_list[2], &mut eval_env);
                    if third.is_error() {
                        return third;
                    } else {
                        return eval_env.set(&second.get_string(), third);
                    }
                } else if *s == "defmacro!" {
                    let second = &uneval_list[1];
                    let mut func = eval(&uneval_list[2], &mut eval_env);
                    func.set_is_macro(true);
                    if func.is_error() {
                        return func;
                    } else {
                        return eval_env.set(&second.get_string(), func);
                    }
                } else if *s == "macroexpand" {
                    return macroexpand(&uneval_list[1], env);
                } else if *s == "let*" {
                    eval_env = new_let_env(&uneval_list[1], &mut eval_env).unwrap();
                    ast = uneval_list[2].clone();
                } else if *s == "quote" {
                    return uneval_list[1].clone();
                } else if *s == "quasiquote" {
                    ast = quasiquote(&uneval_list[1]);
                } else if *s == "do" {
                    let temp = eval_ast(
                        &MalType::list(uneval_list[1..uneval_list.len() - 1].to_vec()),
                        &mut eval_env,
                    );
                    if temp.is_list() {
                        ast = uneval_list[uneval_list.len() - 1].clone();
                    } else {
                        return MalType::error(
                            "Internal Error: eval_ast of list did not return a list".to_string(),
                        );
                    }
                } else if *s == "if" {
                    let temp = eval(&uneval_list[1], &mut eval_env);
                    if temp.is_error() {
                        return temp;
                    } else if temp.is_nil() || (temp.is_bool() && temp.get_bool() == false) {
                        if uneval_list.len() > 3 {
                            ast = uneval_list[3].clone();
                        } else {
                            return MalType::nil();
                        }
                    } else {
                        if uneval_list.len() > 2 {
                            ast = uneval_list[2].clone();
                        } else {
                            return MalType::nil();
                        }
                    }
                } else if *s == "fn*" {
                    if uneval_list[1].is_list() || uneval_list[1].is_vector() {
                        let binds = &*uneval_list[1].get_list();
                        //need to clone everything to prevent dangaling references
                        let binds_clone = binds.clone();
                        let function_body = uneval_list[2].clone();

                        //create new clone environment, to cut ties to passed in env
                        let new_env = eval_env.clone();

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
                        return MalType::func_tco(
                            binds.clone(),
                            Box::new(uneval_list[2].clone()),
                            eval_env.clone(),
                            Rc::new(Box::new(new_func)),
                            false,
                        );
                    } else {
                        return MalType::error(format!(
                            "bind list is not a list: {} ",
                            pr_str(&uneval_list[1], true)
                        ));
                    }
                } else {
                    //fist element in list is a symbol but not a special form
                    //return eval_list(&ast, &mut eval_env);
                    let eval_list_ast = eval_ast(&ast, &mut eval_env);
                    if eval_list_ast.is_list() {
                        let eval_list = eval_list_ast.get_list();
                        let first = &eval_list[0];
                        if first.is_error() {
                            return first.clone();
                        } else if first.is_func() {
                            let (f, _is_macro) = first.get_func();
                            //println!("#1 in MalType::Func(f) = first: {:?}", f);
                            return f(eval_list[1..].to_vec());
                        } else if first.is_func_tco() {
                            let (args, body, env, _func, _is_macro) = first.get_func_tco();
                            ast = *body;
                            let mut new_func_env = env.get_inner();

                            //bind function arguments
                            new_func_env.bind_exprs(&args, &eval_list[1..]);

                            eval_env = new_func_env;
                        } else {
                            return MalType::error(format!("{} not found.", pr_str(first, true)));
                        }
                    } else {
                        return MalType::error(
                            "internal error: eval_ast of List did not return a List".to_string(),
                        );
                    }
                }
            } else {
                //first element is not a symbol, must be a Func or a TCOFunc
                let eval_list_ast = eval_ast(&ast, &mut eval_env);
                if eval_list_ast.is_list() {
                    let eval_list = eval_list_ast.get_list();
                    let first = &eval_list[0];
                    if first.is_error() {
                        return first.clone();
                    } else if first.is_func() {
                        let (f, _is_macro) = first.get_func();
                        //println!("#2 in MalType::Func(f) = first: {:?}", f);
                        return f(eval_list[1..].to_vec());
                    } else if first.is_func_tco() {
                        let (args, body, env, _func, _is_macro) = first.get_func_tco();
                        ast = *body;
                        let mut new_func_env = env.get_inner();

                        //bind function arguments
                        new_func_env.bind_exprs(&args, &eval_list[1..]);

                        eval_env = new_func_env;
                    } else {
                        return MalType::error(format!("{} not found.", pr_str(first, true)));
                    }
                } else {
                    return MalType::error(
                        "internal error: eval_ast of List did not return a List".to_string(),
                    );
                }
            }
        } else {
            //ast is not a list
            return eval_ast(&ast, &mut eval_env);
        }
    }
}

pub fn eval_ast(t: &MalType, env: &mut Environment) -> MalType {
    //println!("eval_ast: {:?}", t);
    if t.is_symbol() {
        let s = t.get_string();
        let lookup = env.get(&s);
        if lookup.is_error() {
            MalType::error(format!("{} not found.", s))
        } else {
            lookup
        }
    } else if t.is_list() {
        let l = t.get_list();
        let new_l: Vec<MalType> = l.iter().map(|item| eval(item, env)).collect();
        MalType::list(new_l)
    } else if t.is_vector() {
        let l = t.get_list();
        let new_l: Vec<MalType> = l.iter().map(|item| eval(item, env)).collect();
        MalType::vector(new_l)
    } else if t.is_map() {
        let l = t.get_list();
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
        MalType::map(new_l)
    } else {
        t.clone()
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

        env.set("key1", MalType::int(1));
        env.set("key2", MalType::int(2));
        env.set("key3", MalType::int(3));

        assert_eq!(env.get("key1"), MalType::int(1));
        assert_eq!(env.get("key2"), MalType::int(2));
        assert_eq!(env.get("key3"), MalType::int(3));
        assert_eq!(
            env.get("won't find"),
            MalType::error("won\'t find not found.".to_string())
        );

        let inner = env.get_inner();
        assert_eq!(inner.get("key1"), MalType::int(1));
        assert_eq!(
            inner.get("won't find"),
            MalType::error("won\'t find not found.".to_string())
        );

        inner.set("key3", MalType::int(33));
        assert_eq!(inner.get("key3"), MalType::int(33));

        let mut inner2 = inner.get_inner();
        assert_eq!(inner2.get("key1"), MalType::int(1));
        assert_eq!(
            inner2.get("won't find"),
            MalType::error("won\'t find not found.".to_string())
        );

        inner2.set("key3", MalType::int(333));
        assert_eq!(inner2.get("key3"), MalType::int(333));

        let mut bind: Vec<MalType> = Vec::new();
        let mut expr: Vec<MalType> = Vec::new();

        bind.push(MalType::symbol("a".to_string()));
        bind.push(MalType::symbol("b".to_string()));
        bind.push(MalType::symbol("c".to_string()));

        expr.push(MalType::int(666));
        expr.push(MalType::int(777));
        expr.push(MalType::int(888));

        inner2.bind_exprs(&bind, &expr);
        assert_eq!(inner2.get("a"), MalType::int(666));
        assert_eq!(inner2.get("b"), MalType::int(777));
        assert_eq!(inner2.get("c"), MalType::int(888));

        env.set("newSymbol", MalType::int(456));
        assert_eq!(inner2.get("newSymbol"), MalType::int(456));

        let new_env = env.clone();
        assert_eq!(new_env.get("newSymbol"), MalType::int(456));

        new_env.set("newSymbol2", MalType::int(9876));
        assert_eq!(inner2.get("newSymbol2"), MalType::int(9876));
    }

    #[test]
    fn eval_test_step2() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let mut tests: Vec<(&str, MalType)> = Vec::new();
        tests.push(("(+ 1 2)", MalType::int(3)));
        tests.push(("(+ 5 (* 2 3))", MalType::int(11)));
        tests.push(("(- (+ 5 (* 2 3)) 3)", MalType::int(8)));
        tests.push(("(/ (- (+ 5 (* 2 3)) 3) 4)", MalType::int(2)));
        tests.push(("(/ (- (+ 515 (* 87 311)) 302) 27)", MalType::int(1010)));
        tests.push(("(* -3 6)", MalType::int(-18)));
        tests.push(("(/ (- (+ 515 (* -87 311)) 296) 27)", MalType::int(-994)));
        tests.push(("(abc 1 2 3)", MalType::error("abc not found.".to_string())));

        let empty_vec: Vec<MalType> = vec![];
        tests.push(("()", MalType::list(empty_vec)));

        let result_vec: Vec<MalType> = vec![MalType::int(1), MalType::int(2), MalType::int(3)];
        tests.push(("[1 2 (+ 1 2)]", MalType::vector(result_vec)));

        let result_vec: Vec<MalType> = vec![MalType::string("a".to_string()), MalType::int(15)];
        tests.push(("{\"a\" (+ 7 8)}", MalType::map(result_vec)));

        let result_vec: Vec<MalType> = vec![MalType::keyword(":a".to_string()), MalType::int(15)];
        tests.push(("{:a (+ 7 8)}", MalType::map(result_vec)));

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
        tests.push(("(+ 1 2)", MalType::int(3)));
        tests.push(("(/ (- (+ 5 (* 2 3)) 3) 4)", MalType::int(2)));
        tests.push(("(def! x 3)", MalType::int(3)));
        tests.push(("x", MalType::int(3)));
        tests.push(("(def! x 4)", MalType::int(4)));
        tests.push(("x", MalType::int(4)));
        tests.push(("(def! y (+ 1 7))", MalType::int(8)));
        tests.push(("y", MalType::int(8)));
        tests.push(("(def! mynum 111)", MalType::int(111)));
        tests.push(("(def! MYNUM 222)", MalType::int(222)));
        tests.push(("mynum", MalType::int(111)));
        tests.push(("MYNUM", MalType::int(222)));
        tests.push(("(abc 1 2 3)", MalType::error("abc not found.".to_string())));
        tests.push(("(def! w 123)", MalType::int(123)));
        tests.push((
            "(def! w (abc))",
            MalType::error("abc not found.".to_string()),
        ));
        tests.push(("(def! w 123)", MalType::int(123)));
        tests.push(("(let* (x 9) x)", MalType::int(9)));
        tests.push(("(let* (z 9) z)", MalType::int(9)));
        tests.push(("x", MalType::int(4)));
        tests.push(("(let* (z (+ 2 3)) (+ 1 z))", MalType::int(6)));
        tests.push(("(let* (p (+ 2 3) q (+ 2 p)) (+ p q))", MalType::int(12)));
        tests.push(("(def! y (let* (z 7) z))", MalType::int(7)));
        tests.push(("y", MalType::int(7)));
        tests.push(("(def! a 4)", MalType::int(4)));
        tests.push(("(let* (q 9) q)", MalType::int(9)));
        tests.push(("(let* (q 9) a)", MalType::int(4)));
        tests.push(("(let* (z 2) (let* (q 9) a))", MalType::int(4)));
        tests.push(("(let* (x 4) (def! a 5))", MalType::int(5)));
        tests.push(("a", MalType::int(4)));
        tests.push(("(let* [z 9] z)", MalType::int(9)));
        tests.push(("(let* [p (+ 2 3) q (+ 2 p)] (+ p q))", MalType::int(12)));

        let mut v1 = Vec::new();
        v1.push(MalType::int(3));
        v1.push(MalType::int(4));
        v1.push(MalType::int(5));
        let mut v2 = Vec::new();
        v2.push(MalType::int(6));
        v2.push(MalType::int(7));
        v1.push(MalType::vector(v2));
        v1.push(MalType::int(8));
        tests.push(("(let* (a 5 b 6) [3 4 a [b 7] 8])", MalType::vector(v1)));

        for tup in tests {
            //println!("{:?}", tup.0);
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
        tests.push(("(list)", MalType::list(Vec::new())));
        tests.push(("(list? (list))", MalType::bool(true)));
        tests.push(("(empty? (list))", MalType::bool(true)));
        tests.push(("(empty? (list 1))", MalType::bool(false)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("(list 1 2 3)", MalType::list(v1)));
        tests.push(("(count (list 1 2 3))", MalType::int(3)));
        tests.push(("(count (list))", MalType::int(0)));
        tests.push(("(count nil)", MalType::int(0)));
        tests.push((
            "(if (> (count (list 1 2 3)) 3) \"yes\" \"no\")",
            MalType::string("no".to_string()),
        ));
        tests.push((
            "(if (>= (count (list 1 2 3)) 3) \"yes\" \"no\")",
            MalType::string("yes".to_string()),
        ));

        //;; Testing if form
        tests.push(("(if true 7 8)", MalType::int(7)));
        tests.push(("(if false 7 8)", MalType::int(8)));
        tests.push(("(if true (+ 1 7) (+ 1 8))", MalType::int(8)));
        tests.push(("(if false (+ 1 7) (+ 1 8))", MalType::int(9)));
        tests.push(("(if nil 7 8)", MalType::int(8)));
        tests.push(("(if 0 7 8)", MalType::int(7)));
        tests.push(("(if \"\" 7 8)", MalType::int(7)));
        tests.push(("(if (list) 7 8)", MalType::int(7)));
        tests.push(("(if (list 1 2 3) 7 8)", MalType::int(7)));
        tests.push(("(= (list) nil)", MalType::bool(false)));

        //;; Testing 1-way if form
        tests.push(("(if false (+ 1 7))", MalType::nil()));
        tests.push(("(if nil 8 7)", MalType::int(7)));
        tests.push(("(if true (+ 1 7))", MalType::int(8)));

        //;; Testing basic conditionals
        tests.push(("(= 2 1)", MalType::bool(false)));
        tests.push(("(= 1 1)", MalType::bool(true)));
        tests.push(("(= 1 2)", MalType::bool(false)));
        tests.push(("(= 1 (+ 1 1))", MalType::bool(false)));
        tests.push(("(= 2 (+ 1 1))", MalType::bool(true)));
        tests.push(("(= nil 1)", MalType::bool(false)));
        tests.push(("(= nil nil)", MalType::bool(true)));
        tests.push(("(> 2 1)", MalType::bool(true)));
        tests.push(("(> 1 1)", MalType::bool(false)));
        tests.push(("(> 1 2)", MalType::bool(false)));
        tests.push(("(>= 2 1)", MalType::bool(true)));
        tests.push(("(>= 1 1)", MalType::bool(true)));
        tests.push(("(>= 1 2)", MalType::bool(false)));
        tests.push(("(< 2 1)", MalType::bool(false)));
        tests.push(("(< 1 1)", MalType::bool(false)));
        tests.push(("(< 1 2)", MalType::bool(true)));
        tests.push(("(<= 2 1)", MalType::bool(false)));
        tests.push(("(<= 1 1)", MalType::bool(true)));
        tests.push(("(<= 1 2)", MalType::bool(true)));

        //;; Testing equality
        tests.push(("(= 1 1)", MalType::bool(true)));
        tests.push(("(= 0 0)", MalType::bool(true)));
        tests.push(("(= 1 0)", MalType::bool(false)));
        tests.push(("(= \"\" \"\")", MalType::bool(true)));
        tests.push(("(= \"abc\" \"abc\")", MalType::bool(true)));
        tests.push(("(= \"abc\" \"\")", MalType::bool(false)));
        tests.push(("(= \"\" \"abc\")", MalType::bool(false)));
        tests.push(("(= \"abc\" \"def\")", MalType::bool(false)));
        tests.push(("(= \"abc\" \"ABC\")", MalType::bool(false)));
        tests.push(("(= true true)", MalType::bool(true)));
        tests.push(("(= false false)", MalType::bool(true)));
        tests.push(("(= nil nil)", MalType::bool(true)));
        tests.push(("(= (list) (list))", MalType::bool(true)));
        tests.push(("(= (list 1 2) (list 1 2))", MalType::bool(true)));
        tests.push(("(= (list 1) (list))", MalType::bool(false)));
        tests.push(("(= (list) (list 1))", MalType::bool(false)));
        tests.push(("(= 0 (list))", MalType::bool(false)));
        tests.push(("(= (list) 0)", MalType::bool(false)));
        tests.push(("(= (list) \"\")", MalType::bool(false)));
        tests.push(("(= \"\" (list))", MalType::bool(false)));

        //;; Testing builtin and user defined functions
        tests.push(("(+ 1 2)", MalType::int(3)));
        tests.push(("( (fn* (a b) (+ b a)) 3 4)", MalType::int(7)));
        tests.push(("( (fn* () 4) )", MalType::int(4)));
        tests.push(("( (fn* (f x) (f x)) (fn* (a) (+ 1 a)) 7)", MalType::int(8)));

        //;; Testing closures
        tests.push(("( ( (fn* (a) (fn* (b) (+ a b))) 5) 7)", MalType::int(12)));

        eval(
            &read_str("(def! gen-plus5 (fn* () (fn* (b) (+ 5 b))))"),
            &mut env,
        );
        eval(&read_str("(def! plus5 (gen-plus5))"), &mut env);
        tests.push(("(plus5 7)", MalType::int(12)));

        eval(
            &read_str("(def! gen-plusX (fn* (x) (fn* (b) (+ x b))))"),
            &mut env,
        );
        eval(&read_str("(def! plus7 (gen-plusX 7))"), &mut env);
        tests.push(("(plus7 8)", MalType::int(15)));

        //;; Testing do form
        tests.push(("(do (prn \"prn output1\"))", MalType::nil()));
        tests.push(("(do (prn \"prn output2\") 7)", MalType::int(7)));
        tests.push((
            "(do (prn \"prn output1\") (prn \"prn output2\") (+ 1 2))",
            MalType::int(3),
        ));
        tests.push(("(do (def! a 6) 7 (+ a 8))", MalType::int(14)));
        tests.push(("a", MalType::int(6)));

        //;; Testing special form case-sensitivity
        eval(&read_str("(def! DO (fn* (a) 7))"), &mut env);
        tests.push(("(DO 3)", MalType::int(7)));

        //;; Testing recursive sumdown function
        eval(
            &read_str("(def! sumdown (fn* (N) (if (> N 0) (+ N (sumdown  (- N 1))) 0)))"),
            &mut env,
        );
        tests.push(("(sumdown 1)", MalType::int(1)));
        tests.push(("(sumdown 2)", MalType::int(3)));
        tests.push(("(sumdown 6)", MalType::int(21)));

        //;; Testing recursive fibonacci function
        eval(
            &read_str("(def! fib (fn* (N) (if (= N 0) 1 (if (= N 1) 1 (+ (fib (- N 1)) (fib (- N 2)))))))"),
            &mut env,
        );
        tests.push(("(fib 1)", MalType::int(1)));
        tests.push(("(fib 2)", MalType::int(2)));
        tests.push(("(fib 4)", MalType::int(5)));
        tests.push(("(fib 10)", MalType::int(89)));

        //;; Testing language defined not function
        tests.push(("(not false)", MalType::bool(true)));
        tests.push(("(not nil)", MalType::bool(true)));
        tests.push(("(not true)", MalType::bool(false)));
        tests.push(("(not \"a\")", MalType::bool(false)));
        tests.push(("(not 0)", MalType::bool(false)));

        //;; Testing string quoting
        tests.push(("\"\"", MalType::string("".to_string())));
        tests.push(("\"abc\"", MalType::string("abc".to_string())));
        tests.push(("\"abc  def\"", MalType::string("abc  def".to_string())));
        tests.push(("\"\\\"\"", MalType::string("\"".to_string())));
        tests.push((
            "\"abc\ndef\nghi\"",
            MalType::string("abc\ndef\nghi".to_string()),
        ));
        tests.push((
            "\"abc\\\\def\\\\ghi\"",
            MalType::string("abc\\def\\ghi".to_string()),
        ));
        tests.push(("\"\\\\n\"", MalType::string("\\n".to_string())));

        //;; Testing pr-str
        tests.push(("(pr-str)", MalType::string("".to_string())));
        tests.push(("(pr-str \"\")", MalType::string("".to_string())));
        tests.push(("(pr-str \"abc\")", MalType::string("\"abc\"".to_string())));
        tests.push((
            "(pr-str \"abc def\" \"ghi jkl\")",
            MalType::string("\"abc def\" \"ghi jkl\"".to_string()),
        ));
        tests.push(("(pr-str \"\\\"\")", MalType::string("\"\\\"\"".to_string())));
        tests.push((
            "(pr-str (list 1 2 \"abc\" \"\\\"\") \"def\")",
            MalType::string("(1 2 \"abc\" \"\\\"\") \"def\"".to_string()),
        ));
        tests.push((
            "(pr-str \"abc\\ndef\\nghi\")",
            MalType::string("\"abc\\ndef\\nghi\"".to_string()),
        ));
        tests.push((
            "(pr-str \"abc\\\\def\\\\ghi\")",
            MalType::string("\"abc\\\\def\\\\ghi\"".to_string()),
        ));
        tests.push(("(pr-str (list))", MalType::string("()".to_string())));

        //;; Testing str
        tests.push(("(str)", MalType::string("".to_string())));
        tests.push(("(str \"\")", MalType::string("".to_string())));
        tests.push(("(str \"abc\")", MalType::string("abc".to_string())));
        tests.push(("(str \"\\\"\")", MalType::string("\"".to_string())));
        tests.push(("(str 1 \"abc\" 3)", MalType::string("1abc3".to_string())));
        tests.push((
            "(str \"abc  def\" \"ghi jkl\")",
            MalType::string("abc  defghi jkl".to_string()),
        ));
        tests.push((
            "(str \"abc\\\\def\\\\ghi\")",
            MalType::string("abc\\def\\ghi".to_string()),
        ));
        tests.push((
            "(str (list 1 2 \"abc\" \"\\\"\") \"def\")",
            MalType::string("(1 2 abc \")def".to_string()),
        ));
        tests.push(("(str (list))", MalType::string("()".to_string())));

        //;; Testing prn
        tests.push(("(prn)", MalType::nil()));
        tests.push(("(prn \"\")", MalType::nil()));
        tests.push(("(prn \"abc\")", MalType::nil()));
        tests.push(("(prn \"abc  def\" \"ghi jkl\")", MalType::nil()));
        tests.push(("(prn \"\\\"\")", MalType::nil()));
        tests.push(("(prn \"abc\ndef\nghi\")", MalType::nil()));
        tests.push(("(prn \"abc\\def\\ghi\")", MalType::nil()));
        tests.push(("(prn (list 1 2 \"abc\" \"\\\"\") \"def\")", MalType::nil()));

        //;; Testing println
        tests.push(("(println)", MalType::nil()));
        tests.push(("(println \"\")", MalType::nil()));
        tests.push(("(println \"abc\")", MalType::nil()));
        tests.push(("(println \"abc  def\" \"ghi jkl\")", MalType::nil()));
        tests.push(("(println \"\\\"\")", MalType::nil()));
        tests.push(("(println \"abc\ndef\nghi\")", MalType::nil()));
        tests.push(("(println \"abc\\def\\ghi\")", MalType::nil()));
        tests.push((
            "(println (list 1 2 \"abc\" \"\\\"\") \"def\")",
            MalType::nil(),
        ));

        //;; Testing keywords
        tests.push(("(= :abc :abc)", MalType::bool(true)));
        tests.push(("(= :abc :def)", MalType::bool(false)));
        tests.push(("(= :abc \":abc\")", MalType::bool(false)));

        //;; Testing vector truthiness
        tests.push(("(if [] 7 8)", MalType::int(7)));

        //;; Testing vector printing
        tests.push((
            "(pr-str [1 2 \"abc\" \"\\\"\"] \"def\")",
            MalType::string("[1 2 \"abc\" \"\\\"\"] \"def\"".to_string()),
        ));
        tests.push(("(pr-str [])", MalType::string("[]".to_string())));
        tests.push((
            "(str [1 2 \"abc\" \"\\\"\"] \"def\")",
            MalType::string("[1 2 abc \"]def".to_string()),
        ));
        tests.push(("(str [])", MalType::string("[]".to_string())));

        //;; Testing vector functions
        tests.push(("(count [1 2 3])", MalType::int(3)));
        tests.push(("(empty? [1 2 3])", MalType::bool(false)));
        tests.push(("(empty? [])", MalType::bool(true)));
        tests.push(("(list? [4 5 6])", MalType::bool(false)));

        //;; Testing vector equality
        tests.push(("(= [] (list))", MalType::bool(true)));
        tests.push(("(= [7 8] [7 8])", MalType::bool(true)));
        tests.push(("(= (list 1 2) [1 2])", MalType::bool(true)));
        tests.push(("(= (list 1) [])", MalType::bool(false)));
        tests.push(("(= [] [1])", MalType::bool(false)));
        tests.push(("(= 0 [])", MalType::bool(false)));
        tests.push(("(= [] 0)", MalType::bool(false)));
        tests.push(("(= [] \"\")", MalType::bool(false)));
        tests.push(("(= \"\" [])", MalType::bool(false)));

        //;; Testing vector parameter lists
        tests.push(("( (fn* [] 4) )", MalType::int(4)));
        tests.push(("( (fn* [f x] (f x)) (fn* [a] (+ 1 a)) 7)", MalType::int(8)));

        //;; Nested vector/list equality
        tests.push(("(= [(list)] (list []))", MalType::bool(true)));
        tests.push((
            "(= [1 2 (list 3 4 [5 6])] (list 1 2 [3 4 (list 5 6)]))",
            MalType::bool(true),
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
        tests.push(("( (fn* (& more) (count more)) 1 2 3)", MalType::int(3)));
        tests.push(("( (fn* (& more) (list? more)) 1 2 3)", MalType::bool(true)));
        tests.push(("( (fn* (& more) (count more)) 1)", MalType::int(1)));
        tests.push(("( (fn* (& more) (count more)) )", MalType::int(0)));
        tests.push(("( (fn* (& more) (list? more)) )", MalType::bool(true)));
        tests.push(("( (fn* (a & more) (list? more)) )", MalType::bool(true)));
        tests.push(("( (fn* (a & more) (count more)) 1 2 3)", MalType::int(2)));
        tests.push(("( (fn* (a & more) (count more)) 1)", MalType::int(0)));
        tests.push(("( (fn* (a & more) (list? more)) 1)", MalType::bool(true)));

        for tup in tests {
            //println!("{:?}",tup.0);
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }

    #[test]
    fn eval_test_step5() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let mut tests: Vec<(&str, MalType)> = Vec::new();

        eval(
            &read_str("(def! sum2 (fn* (n acc) (if (= n 0) acc (sum2 (- n 1) (+ n acc)))))"),
            &mut env,
        );
        eval(
            &read_str("(def! foo (fn* (n) (if (= n 0) 0 (bar (- n 1)))))"),
            &mut env,
        );
        eval(
            &read_str("(def! bar (fn* (n) (if (= n 0) 0 (foo (- n 1)))))"),
            &mut env,
        );

        tests.push(("(def! res2 nil)", MalType::nil()));
        tests.push(("(def! res2 (sum2 10000 0))", MalType::int(50005000)));

        tests.push(("(foo 10000)", MalType::int(0)));

        for tup in tests {
            //println!("{:?}",tup.0);
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }

    #[test]
    fn eval_test_step6() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let mut tests: Vec<(&str, MalType)> = Vec::new();

        //;; Testing that (do (do)) not broken by TCO
        tests.push(("(do (do 1 2))", MalType::int(2)));

        //;; Testing read-string, eval and slurp
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        let mut v2 = Vec::new();
        v2.push(MalType::int(3));
        v2.push(MalType::int(4));
        v1.push(MalType::list(v2));
        v1.push(MalType::nil());
        tests.push(("(read-string \"(1 2 (3 4) nil)\")", MalType::list(v1)));

        let mut v1 = Vec::new();
        v1.push(MalType::symbol("+".to_string()));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("(read-string \"(+ 2 3)\")", MalType::list(v1)));
        tests.push(("(read-string \"7 ;; comment\")", MalType::int(7)));
        tests.push(("(read-string \";; comment\")", MalType::nil()));
        tests.push(("(eval (read-string \"(+ 2 3)\"))", MalType::int(5)));
        tests.push((
            "(slurp \"mal_tests/test.txt\")",
            MalType::string("A line of text\n".to_string()),
        ));

        eval(&read_str("(load-file \"mal_tests/inc.mal\")"), &mut env);
        tests.push(("(inc1 7)", MalType::int(8)));
        tests.push(("(inc2 7)", MalType::int(9)));
        tests.push(("(inc3 9)", MalType::int(12)));

        //;; Testing that *ARGV* exists and is an empty list
        tests.push(("(list? *ARGV*)", MalType::bool(true)));
        tests.push(("*ARGV*", MalType::list(Vec::new())));

        //;; Testing atoms
        eval(&read_str("(def! inc3 (fn* (a) (+ 3 a)))"), &mut env);
        tests.push(("(def! a (atom 2))", MalType::atom(MalType::int(2))));
        tests.push(("(atom? a)", MalType::bool(true)));
        tests.push(("(atom? 1)", MalType::bool(false)));
        tests.push(("(deref a)", MalType::int(2)));
        tests.push(("(reset! a 3)", MalType::int(3)));
        tests.push(("(deref a)", MalType::int(3)));
        tests.push(("(swap! a inc3)", MalType::int(6)));
        tests.push(("(deref a)", MalType::int(6)));
        tests.push(("(swap! a (fn* (a) a))", MalType::int(6)));
        tests.push(("(swap! a (fn* (a) (* 2 a)))", MalType::int(12)));
        tests.push(("(swap! a (fn* (a b) (* a b)) 10)", MalType::int(120)));
        tests.push(("(swap! a + 3)", MalType::int(123)));

        //;; Testing swap!/closure interaction
        eval(&read_str("(def! inc-it (fn* (a) (+ 1 a)))"), &mut env);
        eval(&read_str("(def! atm (atom 7))"), &mut env);
        eval(&read_str("(def! f (fn* () (swap! atm inc-it)))"), &mut env);
        tests.push(("(f)", MalType::int(8)));
        tests.push(("(f)", MalType::int(9)));

        //;; Testing comments in a file
        tests.push((
            "(load-file \"mal_tests/incB.mal\")",
            MalType::string("incB.mal return string".to_string()),
        ));
        tests.push(("(inc4 7)", MalType::int(11)));
        tests.push(("(inc5 7)", MalType::int(12)));

        //;; Testing map literal across multiple lines in a file
        eval(&read_str("(load-file \"mal_tests/incC.mal\")"), &mut env);

        let mut v1 = Vec::new();
        v1.push(MalType::string("a".to_string()));
        v1.push(MalType::int(1));

        tests.push(("mymap", MalType::map(v1)));

        //;; Testing `@` reader macro (short for `deref`)
        eval(&read_str("(def! atm2 (atom 9))"), &mut env);
        tests.push(("@atm2", MalType::int(9)));

        //;; Testing that vector params not broken by TCO
        eval(&read_str("(def! g2 (fn* [] 78))"), &mut env);
        tests.push(("(g2)", MalType::int(78)));
        eval(&read_str("(def! g3 (fn* [a] (+ a 78)))"), &mut env);
        tests.push(("(g3 3)", MalType::int(81)));

        for tup in tests {
            //println!("{:?}", tup.0);
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }

    #[test]
    fn eval_test_step7() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let mut tests: Vec<(&str, MalType)> = Vec::new();

        //;; Testing cons function
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        tests.push(("(cons 1 (list))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        tests.push(("(cons 1 (list 2))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("(cons 1 (list 2 3))", MalType::list(v1)));
        let mut v1 = Vec::new();
        let mut v0 = Vec::new();
        v0.push(MalType::int(1));
        v1.push(MalType::list(v0));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("(cons (list 1) (list 2 3))", MalType::list(v1)));

        eval(&read_str("(def! a (list 2 3))"), &mut env);
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("(cons 1 a)", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("a", MalType::list(v1)));

        //;; Testing concat function
        let v1 = Vec::new();
        tests.push(("(concat)", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        tests.push(("(concat (list 1 2))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        v1.push(MalType::int(4));
        tests.push(("(concat (list 1 2) (list 3 4))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        v1.push(MalType::int(4));
        v1.push(MalType::int(5));
        v1.push(MalType::int(6));
        tests.push((
            "(concat (list 1 2) (list 3 4) (list 5 6))",
            MalType::list(v1),
        ));
        let v1 = Vec::new();
        tests.push(("(concat (concat))", MalType::list(v1)));
        let v1 = Vec::new();
        tests.push(("(concat (list) (list))", MalType::list(v1)));

        eval(&read_str("(def! a1 (list 1 2))"), &mut env);
        eval(&read_str("(def! b1 (list 3 4))"), &mut env);
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        v1.push(MalType::int(4));
        v1.push(MalType::int(5));
        v1.push(MalType::int(6));
        tests.push(("(concat a1 b1 (list 5 6))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        tests.push(("a1", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(3));
        v1.push(MalType::int(4));
        tests.push(("b1", MalType::list(v1)));

        //;; Testing regular quote
        tests.push(("(quote 7)", MalType::int(7)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("(quote (1 2 3))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        let mut v0 = Vec::new();
        v0.push(MalType::int(3));
        v0.push(MalType::int(4));
        v1.push(MalType::list(v0));
        tests.push(("(quote (1 2 (3 4)))", MalType::list(v1)));

        //;; Testing simple quasiquote
        tests.push(("(quasiquote 7)", MalType::int(7)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("(quasiquote (1 2 3))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        let mut v0 = Vec::new();
        v0.push(MalType::int(3));
        v0.push(MalType::int(4));
        v1.push(MalType::list(v0));
        tests.push(("(quasiquote (1 2 (3 4)))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::nil());
        tests.push(("(quasiquote (nil))", MalType::list(v1)));

        //;; Testing unquote
        tests.push(("(quasiquote (unquote 7))", MalType::int(7)));
        tests.push(("(def! a 8)", MalType::int(8)));
        tests.push(("(quasiquote a)", MalType::symbol("a".to_string())));
        tests.push(("(quasiquote (unquote a))", MalType::int(8)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::symbol("a".to_string()));
        v1.push(MalType::int(3));
        tests.push(("(quasiquote (1 a 3))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(8));
        v1.push(MalType::int(3));
        tests.push(("(quasiquote (1 (unquote a) 3))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::string("b".to_string()));
        v1.push(MalType::string("d".to_string()));
        tests.push(("(def! b (quote (1 \"b\" \"d\")))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::symbol("b".to_string()));
        v1.push(MalType::int(3));
        tests.push(("(quasiquote (1 b 3))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        let mut v0 = Vec::new();
        v0.push(MalType::int(1));
        v0.push(MalType::string("b".to_string()));
        v0.push(MalType::string("d".to_string()));
        v1.push(MalType::list(v0));
        v1.push(MalType::int(3));
        tests.push(("(quasiquote (1 (unquote b) 3))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        tests.push(("(quasiquote ((unquote 1) (unquote 2)))", MalType::list(v1)));

        //;; Testing splice-unquote
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::string("b".to_string()));
        v1.push(MalType::string("d".to_string()));
        tests.push(("(def! c (quote (1 \"b\" \"d\")))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::symbol("c".to_string()));
        v1.push(MalType::int(3));
        tests.push(("(quasiquote (1 c 3))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(1));
        v1.push(MalType::string("b".to_string()));
        v1.push(MalType::string("d".to_string()));
        v1.push(MalType::int(3));
        tests.push(("(quasiquote (1 (splice-unquote c) 3))", MalType::list(v1)));

        //;; Testing symbol equality
        tests.push(("(= (quote abc) (quote abc))", MalType::bool(true)));
        tests.push(("(= (quote abc) (quote abcd))", MalType::bool(false)));
        tests.push(("(= (quote abc) \"abc\")", MalType::bool(false)));
        tests.push(("(= \"abc\" (quote abc))", MalType::bool(false)));
        tests.push(("(= \"abc\" (str (quote abc)))", MalType::bool(true)));
        tests.push(("(= (quote abc) nil)", MalType::bool(false)));
        tests.push(("(= nil (quote abc))", MalType::bool(false)));

        //;; Testing ' (quote) reader macro
        tests.push(("'7", MalType::int(7)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("'(1 2 3)", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        let mut v0 = Vec::new();
        v0.push(MalType::int(3));
        v0.push(MalType::int(4));
        v1.push(MalType::list(v0));
        tests.push(("'(1 2 (3 4))", MalType::list(v1)));

        //;; Testing ` (quasiquote) reader macro
        tests.push(("`7", MalType::int(7)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("`(1 2 3)", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        let mut v0 = Vec::new();
        v0.push(MalType::int(3));
        v0.push(MalType::int(4));
        v1.push(MalType::list(v0));
        tests.push(("`(1 2 (3 4))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::nil());
        tests.push(("`(nil)", MalType::list(v1)));

        //;; Testing ~ (unquote) reader macro
        tests.push(("`~7", MalType::int(7)));
        tests.push(("(def! a 8)", MalType::int(8)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(8));
        v1.push(MalType::int(3));
        tests.push(("`(1 ~a 3)", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::string("b".to_string()));
        v1.push(MalType::string("d".to_string()));
        tests.push(("(def! b '(1 \"b\" \"d\"))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::symbol("b".to_string()));
        v1.push(MalType::int(3));
        tests.push(("`(1 b 3)", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        let mut v0 = Vec::new();
        v0.push(MalType::int(1));
        v0.push(MalType::string("b".to_string()));
        v0.push(MalType::string("d".to_string()));
        v1.push(MalType::list(v0));
        v1.push(MalType::int(3));
        tests.push(("`(1 ~b 3)", MalType::list(v1)));

        //;; Testing ~@ (splice-unquote) reader macro
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::string("b".to_string()));
        v1.push(MalType::string("d".to_string()));
        tests.push(("(def! c '(1 \"b\" \"d\"))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::symbol("c".to_string()));
        v1.push(MalType::int(3));
        tests.push(("`(1 c 3)", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(1));
        v1.push(MalType::string("b".to_string()));
        v1.push(MalType::string("d".to_string()));
        v1.push(MalType::int(3));
        tests.push(("`(1 ~@c 3)", MalType::list(v1)));

        //;; Testing cons, concat, first, rest with vectors
        let mut v1 = Vec::new();
        let mut v0 = Vec::new();
        v0.push(MalType::int(1));
        v1.push(MalType::vector(v0));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("(cons [1] [2 3])", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        tests.push(("(cons 1 [2 3])", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(2));
        v1.push(MalType::int(3));
        v1.push(MalType::int(4));
        v1.push(MalType::int(5));
        v1.push(MalType::int(6));
        tests.push(("(concat [1 2] (list 3 4) [5 6])", MalType::list(v1)));

        //;; Testing unquote with vectors
        tests.push(("(def! a 8)", MalType::int(8)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::symbol("a".to_string()));
        v1.push(MalType::int(3));
        tests.push(("`[1 a 3]", MalType::list(v1)));

        //;; Testing splice-unquote with vectors
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::string("b".to_string()));
        v1.push(MalType::string("d".to_string()));
        tests.push(("(def! c '(1 \"b\" \"d\"))", MalType::list(v1)));
        let mut v1 = Vec::new();
        v1.push(MalType::int(1));
        v1.push(MalType::int(1));
        v1.push(MalType::string("b".to_string()));
        v1.push(MalType::string("d".to_string()));
        v1.push(MalType::int(3));
        tests.push(("`[1 ~@c 3]", MalType::list(v1)));

        for tup in tests {
            //println!("{:?}", tup.0);
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }

    #[test]
    fn eval_test_step8() {
        let mut env = Environment::new();
        init_environment(&mut env);

        let mut tests: Vec<(&str, MalType)> = Vec::new();

        //;; Testing trivial macros
        eval(&read_str("(defmacro! one (fn* () 1))"), &mut env);
        eval(&read_str("(defmacro! two (fn* () 2))"), &mut env);
        tests.push(("(one)", MalType::int(1)));
        tests.push(("(two)", MalType::int(2)));

        //;; Testing unless macros
        eval(
            &read_str("(defmacro! unless (fn* (pred a b) `(if ~pred ~b ~a)))"),
            &mut env,
        );
        tests.push(("(unless false 7 8)", MalType::int(7)));
        tests.push(("(unless true 7 8)", MalType::int(8)));
        eval(
            &read_str("(defmacro! unless2 (fn* (pred a b) `(if (not ~pred) ~a ~b)))"),
            &mut env,
        );
        tests.push(("(unless2 false 7 8)", MalType::int(7)));
        tests.push(("(unless2 true 7 8)", MalType::int(8)));

        //;; Testing macroexpand
        let mut v1 = Vec::new();
        v1.push(MalType::symbol("if".to_string()));
        let mut v2 = Vec::new();
        v2.push(MalType::symbol("not".to_string()));
        v2.push(MalType::int(2));
        v1.push(MalType::list(v2));
        v1.push(MalType::int(3));
        v1.push(MalType::int(4));
        tests.push(("(macroexpand (unless2 2 3 4))", MalType::list(v1)));

        for tup in tests {
            println!("{:?}", tup.0);
            let ast = read_str(tup.0);
            assert_eq!(eval(&ast, &mut env), tup.1);
        }
    }
}
