extern crate regex;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod eval;
mod printer;
mod reader;
mod types;

fn rep(line: &str, env: &types::Env) -> String {
    let ast = reader::read_str(line);
    let result = eval::eval(&ast,env);

    printer::pr_str(&result)
}

const HISTORY_FILE: &str = ".history.txt";

fn main() {
    let env = eval::init_repl_env();

    let mut env2 = eval::Environment::new();
    eval::init_environment(&mut env2);

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
                println!("{}", rep(&line,&env));
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
