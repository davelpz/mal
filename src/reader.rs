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
                MalType::Symbol(t.to_string())
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
