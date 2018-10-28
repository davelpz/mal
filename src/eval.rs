use std::collections::HashMap;
use std::rc::Rc;
use types::BuiltinFuncArgs;
use types::Env;
use types::MalError;
use types::MalType;

fn addition_builtin(_args: BuiltinFuncArgs) -> MalType {
    MalType::Int(1)
}

pub fn init_repl_env() -> Env {
    let mut repl_env: Env = HashMap::new();

    repl_env.insert("+".to_string(), Rc::new(Box::new(addition_builtin)));
    repl_env.insert("-".to_string(), Rc::new(Box::new(addition_builtin)));
    repl_env.insert("*".to_string(), Rc::new(Box::new(addition_builtin)));
    repl_env.insert("/".to_string(), Rc::new(Box::new(addition_builtin)));

    repl_env
}

pub fn eval<'a, 'b>(t: &'a MalType, env: &'b Env) -> Result<MalType, MalError> {
    match t {
        MalType::List(l) if l.len() == 0 => Ok(t.clone()),
        MalType::List(l) if l.len() > 0 => {
            let eval_list = eval_ast(t, env).unwrap();
            if let MalType::List(l) = eval_list {
                let first = &l[0];
                if let MalType::Func(f) = first {
                    Ok(f(l[1..].to_vec()))
                } else {
                    Err(MalError::new(
                        "internal error: First element of List is not a function".to_string(),
                    ))
                }
            } else {
                Err(MalError::new(
                    "internal error: eval_ast of List did not return a List".to_string(),
                ))
            }
        }
        _ => eval_ast(t, env),
    }
}

pub fn eval_ast<'a, 'b>(t: &'a MalType, env: &'b Env) -> Result<MalType, MalError> {
    match t {
        MalType::Symbol(s) => {
            let lookup = env.get(s);
            match lookup {
                Some(f) => Ok(MalType::Func(f.clone())),
                None => Err(MalError::new(format!("{} not defined.", s))),
            }
        }
        MalType::List(l) => {
            let new_l: Vec<MalType> = l.into_iter().map(|item| eval(item, env).unwrap()).collect();
            Ok(MalType::List(new_l))
        }
        _ => Ok(t.clone()),
    }
}
