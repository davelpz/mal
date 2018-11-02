use regex::Captures;
use regex::Regex;
use std::str::FromStr;
use types::MalType;

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


pub struct Reader {
    tokens: Vec<String>,
    position: usize,
}

impl Reader {
    fn new(tokens: Vec<String>) -> Reader {
        Reader {
            position: 0,
            tokens,
        }
    }

    fn peek(&self) -> Option<&str> {
        if self.position < self.tokens.len() {
            Some(&self.tokens[self.position])
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<&str> {
        //println!("{:?}", self.position);

        if self.position < self.tokens.len() {
            let current = &self.tokens[self.position];
            self.position += 1;
            Some(current)
        } else {
            None
        }
    }
}

fn is_close_char_or_end(reader: &mut Reader, close_char: &str) -> bool {
    match reader.peek() {
        Some(tok) => tok == close_char,
        None => true,
    }
}

fn read_list(reader: &mut Reader, close_char: &str) -> Vec<MalType> {
    //println!("read_list: {:?}", reader.peek());
    reader.next(); //need to eat the opening paren

    let mut l: Vec<MalType> = Vec::new();

    while !is_close_char_or_end(reader, close_char) {
        l.push(read_form(reader));
    }

    reader.next(); //need to eat the closing paren

    //println!("{:?}", l);
    l
}

fn parsable<T: FromStr>(s: &str) -> bool {
    s.parse::<T>().is_ok()
}

fn unescape_str(s: &str) -> String {
    let re: Regex = Regex::new(r#"\\(.)"#).unwrap();
    re.replace_all(&s, |caps: &Captures| {
        if &caps[1] == "n" {
            "\n"
        } else if &caps[1] == "t" {
            "\t"
        } else if &caps[1] == "\\" {
            "\\"
        } else {
            &caps[1]
        }.to_string()
    }).to_string()
}

fn read_atom(reader: &mut Reader) -> MalType {
    //println!("read_atom: {:?}", reader.peek());
    match reader.next() {
        Some(t) if parsable::<i64>(t) => MalType::Int(t.parse().unwrap()),
        Some(t) if parsable::<f64>(t) => MalType::Float(t.parse().unwrap()),
        Some(t) if parsable::<bool>(t) => MalType::Bool(t.parse().unwrap()),
        Some(t) => {
            let first_char = t.chars().next().unwrap();
            if first_char == '\"' {
                //MalType::Str(t.to_string())
                MalType::Str(unescape_str(t))
            } else if first_char == ':' {
                MalType::KeyWord(t.to_string())
            } else if t == "nil" {
                MalType::Nil
            } else {
                MalType::Symbol(t.to_string())
            }
        }
        _ => MalType::Nil,
    }
}

fn make_quote_list(quote: String, reader: &mut Reader) -> MalType {
    reader.next(); //eat the quote
    let next_form = read_form(reader);
    let mut v: Vec<MalType> = Vec::new();
    v.push(MalType::Symbol(quote));
    v.push(next_form);
    MalType::List(v)
}

fn make_meta_list(reader: &mut Reader) -> MalType {
    reader.next(); //eat the quote
    let meta_form = read_form(reader);
    let next_form = read_form(reader);
    let mut v: Vec<MalType> = Vec::new();
    v.push(MalType::Symbol("with-meta".to_string()));
    v.push(next_form);
    v.push(meta_form);
    MalType::List(v)
}

pub fn read_form(reader: &mut Reader) -> MalType {
    //println!("read_form: {:?}", reader.peek());
    match reader.peek() {
        Some(TOKEN_LEFT_PAREN) => MalType::List(read_list(reader, TOKEN_RIGHT_PAREN)),
        Some(TOKEN_LEFT_BRACKET) => {
            MalType::Vector(read_list(reader, TOKEN_RIGHT_BRACKET))
        }
        Some(TOKEN_LEFT_CURLY) => MalType::Map(read_list(reader, TOKEN_RIGHT_CURLY)),
        Some(TOKEN_QUOTE) => make_quote_list("quote".to_string(), reader),
        Some(TOKEN_QUASIQUOTE) => make_quote_list("quasiquote".to_string(), reader),
        Some(TOKEN_UNQUOTE) => make_quote_list("unquote".to_string(), reader),
        Some(TOKEN_SPLICE_UNQUOTE) => make_quote_list("splice-unquote".to_string(), reader),
        Some(TOKEN_DEREF) => make_quote_list("deref".to_string(), reader),
        Some(TOKEN_WITH_META) => make_meta_list(reader),
        Some(_) => read_atom(reader),
        None => MalType::Nil,
    }
}

pub fn read_str(line: &str) -> MalType {
    let mut r = Reader::new(tokenizer(line));
    read_form(&mut r)
}

pub fn tokenizer(line: &str) -> Vec<String> {
    let re: Regex =
        Regex::new(r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"###)
            .unwrap();
    let mut v: Vec<String> = Vec::new();

    for caps in re.captures_iter(line) {
        let token_str = caps.get(1).map_or("", |m| m.as_str());

        //ignore commments that start with ;
        if !token_str.starts_with(';') {
            v.push(token_str.to_string());
        }
        //println!("{:?}\n", token_str)
    }

    //println!("{:?}", v);
    v
}

/*
  Unit Tests for various functions/methods
*/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer_test() {
        assert_eq!(Vec::<String>::new(), tokenizer(""));
        assert_eq!(Vec::<String>::new(), tokenizer("\n"));
        assert_eq!(vec!["nil"], tokenizer("nil"));
        assert_eq!(vec!["123"], tokenizer("123"));
        assert_eq!(vec!["(", ")"], tokenizer("()"));
        assert_eq!(vec!["[", "]"], tokenizer("[]"));
        assert_eq!(vec!["{", "}"], tokenizer("{}"));
        assert_eq!(vec!["\"abc\""], tokenizer("\"abc\""));
        //assert_eq!(vec!["~@(", "1", "2", "3", ")"], tokenizer("~@(1 2 3)"));
        assert_eq!(Vec::<String>::new(), tokenizer(";this is a test"));
        assert_eq!(
            vec!["(", "+", "1", "a", ")"],
            tokenizer(" (+ 1 a) ;this is a test")
        );
        assert_eq!(
            vec!["(", "-", "(", "+", "1", "a", ")", "234.3", ")"],
            tokenizer("(- (+ 1 a) 234.3);this is a test")
        );
    }

    #[test]
    fn reader_test() {
        let mut r = Reader::new(tokenizer("(- (+ 1 a) 234.3);this is a test"));
        assert_eq!(Some("("), r.peek());
        assert_eq!(Some("("), r.next());
        assert_eq!(Some("-"), r.peek());
        assert_eq!(Some("-"), r.next());
        assert_eq!(Some("("), r.peek());
        assert_eq!(Some("("), r.next());
        assert_eq!(Some("+"), r.peek());
        assert_eq!(Some("+"), r.next());
        assert_eq!(Some("1"), r.peek());
        assert_eq!(Some("1"), r.next());
        assert_eq!(Some("a"), r.peek());
        assert_eq!(Some("a"), r.next());
        assert_eq!(Some(")"), r.peek());
        assert_eq!(Some(")"), r.next());
        assert_eq!(Some("234.3"), r.peek());
        assert_eq!(Some("234.3"), r.next());
        assert_eq!(Some(")"), r.peek());
        assert_eq!(Some(")"), r.next());
        assert_eq!(None, r.peek());
        assert_eq!(None, r.next());
    }

    #[test]
    fn read_atom_test() {
        let mut r = Reader::new(tokenizer(
            "(- (+ 1 a) 234.3 :kw1 nil \"boo\" true false);this is a test",
        ));
        assert_eq!(MalType::Symbol("(".to_string()), read_atom(&mut r));
        assert_eq!(MalType::Symbol("-".to_string()), read_atom(&mut r));
        assert_eq!(MalType::Symbol("(".to_string()), read_atom(&mut r));
        assert_eq!(MalType::Symbol("+".to_string()), read_atom(&mut r));
        assert_eq!(MalType::Int(1), read_atom(&mut r));
        assert_eq!(MalType::Symbol("a".to_string()), read_atom(&mut r));
        assert_eq!(MalType::Symbol(")".to_string()), read_atom(&mut r));
        assert_eq!(MalType::Float(234.3), read_atom(&mut r));
        assert_eq!(MalType::KeyWord(":kw1".to_string()), read_atom(&mut r));
        assert_eq!(MalType::Nil, read_atom(&mut r));
        assert_eq!(MalType::Str("\"boo\"".to_string()), read_atom(&mut r));
        assert_eq!(MalType::Bool(true), read_atom(&mut r));
        assert_eq!(MalType::Bool(false), read_atom(&mut r));
        assert_eq!(MalType::Symbol(")".to_string()), read_atom(&mut r));
        assert_eq!(MalType::Nil, read_atom(&mut r));
    }

    #[test]
    fn read_form_test() {
        let mut r = Reader::new(tokenizer("(- (+ 1 a) 234.3 \"boo\" :akeyword)"));
        let mut v1: Vec<MalType> = Vec::new();
        let mut v2: Vec<MalType> = Vec::new();
        v2.push(MalType::Symbol("+".to_string()));
        v2.push(MalType::Int(1));
        v2.push(MalType::Symbol("a".to_string()));

        v1.push(MalType::Symbol("-".to_string()));
        v1.push(MalType::List(v2));
        v1.push(MalType::Float(234.3));
        v1.push(MalType::Str("\"boo\"".to_string()));
        v1.push(MalType::KeyWord(":akeyword".to_string()));

        assert_eq!(MalType::List(v1), read_form(&mut r));

        r = Reader::new(tokenizer("'1"));
        v1 = Vec::new();
        v1.push(MalType::Symbol("quote".to_string()));
        v1.push(MalType::Int(1));
        assert_eq!(MalType::List(v1), read_form(&mut r));

        r = Reader::new(tokenizer("`1"));
        v1 = Vec::new();
        v1.push(MalType::Symbol("quasiquote".to_string()));
        v1.push(MalType::Int(1));
        assert_eq!(MalType::List(v1), read_form(&mut r));

        r = Reader::new(tokenizer("~1"));
        v1 = Vec::new();
        v1.push(MalType::Symbol("unquote".to_string()));
        v1.push(MalType::Int(1));
        assert_eq!(MalType::List(v1), read_form(&mut r));

        r = Reader::new(tokenizer("~@1"));
        v1 = Vec::new();
        v1.push(MalType::Symbol("splice-unquote".to_string()));
        v1.push(MalType::Int(1));
        assert_eq!(MalType::List(v1), read_form(&mut r));

        r = Reader::new(tokenizer("@1"));
        v1 = Vec::new();
        v1.push(MalType::Symbol("deref".to_string()));
        v1.push(MalType::Int(1));
        assert_eq!(MalType::List(v1), read_form(&mut r));

        r = Reader::new(tokenizer("^{\"a\" 1} [1 2 3]"));
        v1 = Vec::new();
        v2 = Vec::new();
        let mut v3: Vec<MalType> = Vec::new();

        v2.push(MalType::Int(1));
        v2.push(MalType::Int(2));
        v2.push(MalType::Int(3));

        v3.push(MalType::Str("\"a\"".to_string()));
        v3.push(MalType::Int(1));

        v1.push(MalType::Symbol("with-meta".to_string()));
        v1.push(MalType::Vector(v2));
        v1.push(MalType::Map(v3));
        assert_eq!(MalType::List(v1), read_form(&mut r));
    }

    #[test]
    fn read_str_test() {
        let mut v1: Vec<MalType> = Vec::new();
        let mut v2: Vec<MalType> = Vec::new();
        v2.push(MalType::Symbol("+".to_string()));
        v2.push(MalType::Int(1));
        v2.push(MalType::Symbol("a".to_string()));

        v1.push(MalType::Symbol("-".to_string()));
        v1.push(MalType::List(v2));
        v1.push(MalType::Float(234.3));
        v1.push(MalType::Str("\"boo\"".to_string()));

        assert_eq!(MalType::List(v1), read_str("(- (+ 1 a) 234.3 \"boo\")"));
    }
}
