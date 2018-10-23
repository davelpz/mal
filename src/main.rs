extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod parse;
mod eval;
mod print;

fn rep(line: &str) -> &str {
    print::print(eval::eval(parse::read(line)))
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
