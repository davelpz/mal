extern crate rustyline;
extern crate regex;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod reader;
mod eval;
mod printer;
mod types;

fn rep(line: &str) -> String {
    printer::pr_str(eval::eval(&reader::read_str(line)))
}

const HISTORY_FILE: &str = ".history.txt";

fn main() {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history(HISTORY_FILE).is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                println!("{}", rep(&line));
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(HISTORY_FILE).unwrap();
}
