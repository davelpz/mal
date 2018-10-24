
pub const TOKEN_LEFT_PAREN: &str = "(";
pub const TOKEN_RIGHT_PAREN: &str = ")";

#[derive(Debug)]
#[derive(PartialEq)]
pub enum MalType {
    Nil,
    Int(i64),
    Float(f64),
    Str(String),
    Symbol(String),
    List(Vec<MalType>)
}