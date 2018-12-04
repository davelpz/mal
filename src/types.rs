use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use eval::Environment;

#[derive(Debug, PartialEq, Clone)]
pub struct MalType {
    pub val: Rc<RefCell<MalEnum>>,
}

#[derive(Debug, PartialEq)]
pub enum MalEnum {
    Nil,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Rc<String>),
    Symbol(Rc<String>),
    KeyWord(Rc<String>),
    Atom(MalType),
    List(Rc<Vec<MalType>>),
    Vector(Rc<Vec<MalType>>),
    Map(Rc<Vec<MalType>>),
    Func(Rc<Box<BuiltinFunc>>, bool),
    TCOFunc(
        Vec<MalType>,
        Box<MalType>,
        Environment,
        Rc<Box<BuiltinFunc>>,
        bool,
    ),
    Error(Rc<String>),
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
    pub fn nil() -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Nil)),
        }
    }
    pub fn int(val: i64) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Int(val))),
        }
    }
    pub fn float(val: f64) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Float(val))),
        }
    }
    pub fn bool(val: bool) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Bool(val))),
        }
    }
    pub fn string(val: String) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Str(Rc::new(val)))),
        }
    }
    pub fn symbol(val: String) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Symbol(Rc::new(val)))),
        }
    }
    pub fn keyword(val: String) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::KeyWord(Rc::new(val)))),
        }
    }
    pub fn atom(val: MalType) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Atom(val))),
        }
    }
    pub fn list(val: Vec<MalType>) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::List(Rc::new(val)))),
        }
    }
    pub fn vector(val: Vec<MalType>) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Vector(Rc::new(val)))),
        }
    }
    pub fn map(val: Vec<MalType>) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Map(Rc::new(val)))),
        }
    }
    pub fn func(f: Rc<Box<BuiltinFunc>>, is_macro: bool) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Func(f, is_macro))),
        }
    }
    pub fn func_tco(
        args: Vec<MalType>,
        body: Box<MalType>,
        env: Environment,
        func: Rc<Box<BuiltinFunc>>,
        is_macro: bool,
    ) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::TCOFunc(
                args, body, env, func, is_macro,
            ))),
        }
    }
    pub fn error(val: String) -> MalType {
        MalType {
            val: Rc::new(RefCell::new(MalEnum::Error(Rc::new(val)))),
        }
    }
    pub fn is_nil(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Nil => true,
            _ => false,
        }
    }
    pub fn is_int(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Int(_) => true,
            _ => false,
        }
    }
    pub fn is_float(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Float(_) => true,
            _ => false,
        }
    }
    pub fn is_bool(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Bool(_) => true,
            _ => false,
        }
    }
    pub fn is_string(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Str(_) => true,
            _ => false,
        }
    }
    pub fn is_symbol(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Symbol(_) => true,
            _ => false,
        }
    }
    pub fn is_keyword(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::KeyWord(_) => true,
            _ => false,
        }
    }
    pub fn is_atom(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Atom(_) => true,
            _ => false,
        }
    }
    pub fn is_list(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::List(_) => true,
            _ => false,
        }
    }
    pub fn is_vector(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Vector(_) => true,
            _ => false,
        }
    }
    pub fn is_map(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Map(_) => true,
            _ => false,
        }
    }
    pub fn is_func(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Func(_, _) => true,
            _ => false,
        }
    }
    pub fn is_func_tco(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::TCOFunc(_, _, _, _, _) => true,
            _ => false,
        }
    }
    pub fn is_macro(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Func(_, is_macro) => is_macro,
            MalEnum::TCOFunc(_, _, _, _, is_macro) => is_macro,
            _ => false,
        }
    }
    pub fn is_error(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Error(_) => true,
            _ => false,
        }
    }
    pub fn get_int(&self) -> i64 {
        match *self.val.borrow() {
            MalEnum::Int(i) => i,
            MalEnum::Float(i) => i as i64,
            _ => panic!(),
        }
    }
    pub fn get_float(&self) -> f64 {
        match *self.val.borrow() {
            MalEnum::Int(i) => i as f64,
            MalEnum::Float(i) => i,
            _ => panic!(),
        }
    }
    pub fn get_bool(&self) -> bool {
        match *self.val.borrow() {
            MalEnum::Bool(b) => b,
            _ => panic!(),
        }
    }
    pub fn get_string(&self) -> Rc<String> {
        let val = self.val.borrow();
        match *val {
            MalEnum::Str(ref s) => s.clone(),
            MalEnum::Symbol(ref s) => s.clone(),
            MalEnum::KeyWord(ref s) => s.clone(),
            MalEnum::Error(ref s) => s.clone(),
            _ => panic!(),
        }
    }
    pub fn get_atom(&self) -> MalType {
        match *self.val.borrow() {
            MalEnum::Atom(ref a) => a.clone(),
            _ => panic!(),
        }
    }
    pub fn get_list(&self) -> Rc<Vec<MalType>> {
        let val = self.val.borrow();
        match *val {
            MalEnum::List(ref l) => l.clone(),
            MalEnum::Vector(ref l) => l.clone(),
            MalEnum::Map(ref l) => l.clone(),
            _ => panic!(),
        }
    }
    pub fn get_func(&self) -> (Rc<Box<BuiltinFunc>>, bool) {
        match *self.val.borrow() {
            MalEnum::Func(ref f, ref is_macro) => (f.clone(), is_macro.clone()),
            _ => panic!(),
        }
    }
    pub fn get_func_tco(
        &self,
    ) -> (
        Vec<MalType>,
        Box<MalType>,
        Environment,
        Rc<Box<BuiltinFunc>>,
        bool,
    ) {
        match self.val.borrow().clone() {
            MalEnum::TCOFunc(a, b, c, f, is_macro) => (a, b, c, f, is_macro),
            _ => panic!(),
        }
    }
    pub fn set_is_macro(&mut self, val: bool) {
        let temp = &mut *self.val.borrow_mut();
        if let MalEnum::Func(_, ref mut is_macro) = temp {
            *is_macro = val;
        } else if let MalEnum::TCOFunc(_, _, _, _, ref mut is_macro) = temp {
            *is_macro = val;
        }
    }
    pub fn set_atom(&mut self, val: MalType) {
        if let MalEnum::Atom(ref mut x) = *self.val.borrow_mut() {
            *x = val;
        }
    }
}

impl Clone for MalEnum {
    fn clone(&self) -> MalEnum {
        match self {
            MalEnum::Nil => MalEnum::Nil,
            MalEnum::Int(i) => MalEnum::Int(*i),
            MalEnum::Float(f) => MalEnum::Float(*f),
            MalEnum::Bool(b) => MalEnum::Bool(*b),
            MalEnum::Str(s) => MalEnum::Str(s.clone()),
            MalEnum::Symbol(s) => MalEnum::Symbol(s.clone()),
            MalEnum::KeyWord(s) => MalEnum::KeyWord(s.clone()),
            MalEnum::Atom(s) => MalEnum::Atom(s.clone()),
            MalEnum::List(l) => MalEnum::List(l.clone()),
            MalEnum::Vector(l) => MalEnum::Vector(l.clone()),
            MalEnum::Map(l) => MalEnum::Map(l.clone()),
            MalEnum::Func(f, is_macro) => MalEnum::Func(f.clone(), *is_macro),
            MalEnum::Error(s) => MalEnum::Error(s.clone()),
            MalEnum::TCOFunc(args, body, env, func, is_macro) => MalEnum::TCOFunc(
                args.clone(),
                body.clone(),
                env.clone(),
                func.clone(),
                *is_macro,
            ),
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
