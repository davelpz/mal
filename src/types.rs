pub const TOKEN_LEFT_PAREN: &str = "(";
pub const TOKEN_RIGHT_PAREN: &str = ")";
pub const TOKEN_LEFT_BRACKET: &str = "[";
pub const TOKEN_RIGHT_BRACKET: &str = "]";
pub const TOKEN_LEFT_CURLY: &str = "{";
pub const TOKEN_RIGHT_CURLY: &str = "}";

#[derive(Debug, PartialEq)]
pub enum MalType {
    Nil,
    Int(i64),
    Float(f64),
    Str(String),
    Symbol(String),
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Map(Vec<MalType>),
}
