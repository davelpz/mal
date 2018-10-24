use regex::Regex;
use std::str::FromStr;
use types;
use types::MalType;

pub struct Reader {
    tokens: Vec<String>,
    position: usize,
}

impl Reader {
    fn new(tokens: Vec<String>) -> Reader {
        Reader {
            position: 0,
            tokens: tokens,
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

fn is_right_paren_or_end(reader: &mut Reader) -> bool {
    match reader.peek() {
        Some(tok) => {
            if tok == types::TOKEN_RIGHT_PAREN {
                true
            } else {
                false
            }
        }
        None => true,
    }
}

pub fn read_list(reader: &mut Reader) -> MalType {
    //println!("read_list: {:?}", reader.peek());
    reader.next(); //need to eat the opening paren

    let mut l: Vec<MalType> = Vec::new();

    while !is_right_paren_or_end(reader) {
        l.push(read_form(reader));
    }

    reader.next(); //need to eat the closing paren

    //println!("{:?}", l);
    MalType::List(l)
}

fn parsable<T: FromStr>(s: &str) -> bool {
    s.parse::<T>().is_ok()
}

pub fn read_atom(reader: &mut Reader) -> MalType {
    //println!("read_atom: {:?}", reader.peek());
    let result = match reader.next() {
        Some(t) if parsable::<i64>(t) => MalType::Int(t.parse().unwrap()),
        Some(t) if parsable::<f64>(t) => MalType::Float(t.parse().unwrap()),
        Some(t) => {
            if t.chars().next().unwrap() == '\"' {
                MalType::Str(t.to_string())
            } else {
                if t == "Nil" {
                    MalType::Nil
                } else {
                    MalType::Symbol(t.to_string())
                }
            }
        }
        _ => MalType::Nil,
    };
    //println!("{:?}", result);
    result
}

pub fn read_form(reader: &mut Reader) -> MalType {
    //println!("read_form: {:?}", reader.peek());
    match reader.peek() {
        Some(types::TOKEN_LEFT_PAREN) => read_list(reader),
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
        v.push(token_str.to_string());
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
        assert_eq!(vec!["Nil"], tokenizer("Nil"));
        assert_eq!(vec!["123"], tokenizer("123"));
        assert_eq!(vec!["(", ")"], tokenizer("()"));
        assert_eq!(vec!["[", "]"], tokenizer("[]"));
        assert_eq!(vec!["{", "}"], tokenizer("{}"));
        assert_eq!(vec!["\"abc\""], tokenizer("\"abc\""));
        assert_eq!(vec![";this is a test"], tokenizer(";this is a test"));
        assert_eq!(
            vec!["(", "+", "1", "a", ")", ";this is a test"],
            tokenizer(" (+ 1 a) ;this is a test")
        );
        assert_eq!(
            vec!["(", "-", "(", "+", "1", "a", ")", "234.3", ")", ";this is a test"],
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
        assert_eq!(Some(";this is a test"), r.peek());
        assert_eq!(Some(";this is a test"), r.next());
        assert_eq!(None, r.peek());
        assert_eq!(None, r.next());
    }

    #[test]
    fn read_atom_test() {
        let mut r = Reader::new(tokenizer("(- (+ 1 a) 234.3 \"boo\");this is a test"));
        assert_eq!(MalType::Symbol("(".to_string()),read_atom(&mut r));
        assert_eq!(MalType::Symbol("-".to_string()),read_atom(&mut r));
        assert_eq!(MalType::Symbol("(".to_string()),read_atom(&mut r));
        assert_eq!(MalType::Symbol("+".to_string()),read_atom(&mut r));
        assert_eq!(MalType::Int(1),read_atom(&mut r));
        assert_eq!(MalType::Symbol("a".to_string()),read_atom(&mut r));
        assert_eq!(MalType::Symbol(")".to_string()),read_atom(&mut r));
        assert_eq!(MalType::Float(234.3),read_atom(&mut r));
        assert_eq!(MalType::Str("\"boo\"".to_string()),read_atom(&mut r));
        assert_eq!(MalType::Symbol(")".to_string()),read_atom(&mut r));
        assert_eq!(MalType::Symbol(";this is a test".to_string()),read_atom(&mut r));
        assert_eq!(MalType::Nil,read_atom(&mut r));
    }

    #[test]
    fn read_form_test() {
        let mut r = Reader::new(tokenizer("(- (+ 1 a) 234.3 \"boo\")"));
        let mut v1: Vec<MalType> = Vec::new();
        let mut v2: Vec<MalType> = Vec::new();
        v2.push(MalType::Symbol("+".to_string()));
        v2.push(MalType::Int(1));
        v2.push(MalType::Symbol("a".to_string()));

        v1.push(MalType::Symbol("-".to_string()));
        v1.push(MalType::List(v2));
        v1.push(MalType::Float(234.3));
        v1.push(MalType::Str("\"boo\"".to_string()));

        assert_eq!(MalType::List(v1),read_form(&mut r));
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

        assert_eq!(MalType::List(v1),read_str("(- (+ 1 a) 234.3 \"boo\")"));
    }
}
