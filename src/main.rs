#[macro_use]
extern crate lazy_static;
extern crate rustyline;

mod builtins;
mod context;
mod error;
#[macro_use]
mod macro_defs;
mod sexpr;

use std::io;
use std::io::Write;

use rustyline::Editor;
use rustyline::error::ReadlineError;

use crate::context::EvalContext;
use crate::sexpr::SExpr;

fn main() -> io::Result<()> {
    let mut rl = Editor::<()>::new();
    let mut eval_context = EvalContext::new();
    loop {
        let line = match rl.readline("> ") {
            Ok(line) => line,
            Err(ReadlineError::Eof) => break,
            _ => panic!("oops unexpected error"),
        };
        rl.add_history_entry(&line);

        let tokens = SExpr::lex(line.as_str());
        let mut tokens_iter = tokens.into_iter();
        while let Some(expr) = SExpr::parse(&mut tokens_iter) {
            match expr.eval(&mut eval_context) {
                Err(e) => {
                    println!("encountered error: {}", e.message());
                },
                Ok(result) => println!("{}", result),
            }
        }
    }

    Ok(())
}
