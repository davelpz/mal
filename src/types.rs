pub const TOKEN_LEFT_PAREN: &str = "(";
pub const TOKEN_RIGHT_PAREN: &str = ")";
pub const TOKEN_LEFT_BRACKET: &str = "[";
pub const TOKEN_RIGHT_BRACKET: &str = "]";
pub const TOKEN_LEFT_CURLY: &str = "{";
pub const TOKEN_RIGHT_CURLY: &str = "}";
pub const TOKEN_QUOTE: &str = "'";
pub const TOKEN_QUASIQUOTE: &str = "`";
pub const TOKEN_UNQUOTE: &str = "~";
pub const TOKEN_SPLICE_UNQUOTE: &str = "~@";
pub const TOKEN_DEREF: &str = "@";
pub const TOKEN_WITH_META: &str = "^";

#[derive(Debug, PartialEq)]
pub enum MalType {
    Nil,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Symbol(String),
    KeyWord(String),
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Map(Vec<MalType>),
}