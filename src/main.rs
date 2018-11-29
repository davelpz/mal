extern crate regex;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;

mod core;
mod eval;
mod printer;
mod reader;
mod types;

use types::MalType;

pub fn rep(line: &str, env: &mut eval::Environment) -> String {
    let ast = reader::read_str(line);
    let result = eval::eval(&ast, env);

    printer::pr_str(&result, true)
}

const HISTORY_FILE: &str = ".history.txt";

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut env = eval::Environment::new();
    core::init_environment(&mut env);

    if args.len() > 1 {
        let file = &args[1];
        let mut argv: Vec<MalType> = Vec::new();

        for arg in &args[2..] {
            argv.push(MalType::string(arg.clone()));
        }

        env.set("*ARGV*".to_string(), MalType::list(argv));

        rep(&format!("(load-file \"{}\"", file), &mut env);
    } else {
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
                    println!("{}", rep(&line, &mut env));
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
}
