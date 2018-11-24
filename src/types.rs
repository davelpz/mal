
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use eval::Environment;

#[derive(Debug, PartialEq)]
pub enum MalType {
    Nil,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Symbol(String),
    KeyWord(String),
    Atom(Rc<RefCell<MalType>>),
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Map(Vec<MalType>),
    Func(Rc<Box<BuiltinFunc>>),
    TCOFunc(Vec<MalType>,Box<MalType>,Environment,Rc<Box<BuiltinFunc>>),
    Error(String),
}

pub type BuiltinFuncArgs = Vec<MalType>;
pub type BuiltinFunc = Fn(BuiltinFuncArgs) -> MalType;

impl fmt::Debug for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BuiltinFunc")
    }
}

impl PartialEq for BuiltinFunc {
    fn eq(&self, _other: &BuiltinFunc) -> bool {
        false
    }
}

impl MalType {
    pub fn get_int(&self) -> i64 {
        match self {
            MalType::Int(i) => *i,
            _ => panic!(),
        }
    }
    pub fn get_float(&self) -> f64 {
        match self {
            MalType::Int(i) => *i as f64,
            MalType::Float(i) => *i,
            _ => panic!(),
        }
    }
    pub fn get_symbol_string(&self) -> String {
        match self {
            MalType::Symbol(s) => s.clone(),
            _ => panic!(),
        }
    }
}

impl Clone for MalType {
    fn clone(&self) -> MalType {
        match self {
            MalType::Nil => MalType::Nil,
            MalType::Int(i) => MalType::Int(*i),
            MalType::Float(f) => MalType::Float(*f),
            MalType::Bool(b) => MalType::Bool(*b),
            MalType::Str(s) => MalType::Str(s.clone()),
            MalType::Symbol(s) => MalType::Symbol(s.clone()),
            MalType::KeyWord(s) => MalType::KeyWord(s.clone()),
            MalType::Atom(s) => MalType::Atom(s.clone()),
            MalType::List(l) => MalType::List(l.clone()),
            MalType::Vector(l) => MalType::Vector(l.clone()),
            MalType::Map(l) => MalType::Map(l.clone()),
            MalType::Func(f) => MalType::Func(f.clone()),
            MalType::Error(s) => MalType::Error(s.clone()),
            MalType::TCOFunc(args,body,env,func) => MalType::TCOFunc(args.clone(),body.clone(),env.clone(),func.clone()),
        }
    }
}

/* Not sure if we need this custom error type
//Defining Error type for mal
#[derive(Debug, Clone)]
pub struct MalError {
    pub description: String,
}

impl MalError {
    pub fn new(description: String) -> MalError {
        MalError { description }
    }
}

impl fmt::Display for MalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for MalError {
    fn description(&self) -> &str {
        &self.description
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
*/
