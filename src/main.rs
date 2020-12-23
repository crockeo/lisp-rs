#[macro_use]
extern crate lazy_static;

mod builtins;
mod context;
mod error;
#[macro_use]
mod macro_defs;
mod sexpr;

use std::io;
use std::io::Write;

use crate::context::EvalContext;
use crate::sexpr::SExpr;

// TODO: decomp all of these components into their own homes; SExpr into their place, EvalContext
// into its own place (maybe along with the eval function on the SExpr struct), etc.

fn main() -> io::Result<()> {
    let mut eval_context = EvalContext::new();
    loop {
        let line = read_line("> ")?;
        if let Some(line) = line {
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
        } else {
            break;
        }
    }

    println!();
    Ok(())
}

/// read_line reads a single line of input from the user and returns it as an owned string WITHOUT
/// newline.
///
/// Ok(None) corresponds to an EOF that was correctly read.
fn read_line(prompt: &str) -> io::Result<Option<String>> {
    let mut line = String::new();
    io::stdout().write(prompt.as_bytes())?;
    io::stdout().flush()?;

    let bytes_read = io::stdin().read_line(&mut line)?;
    if bytes_read == 0 {
        Ok(None)
    } else {
        while let Some(c) = line.chars().next_back() {
            if c == '\n' || c == '\r' {
                line.pop();
            } else {
                break;
            }
        }

        Ok(Some(line))
    }
}

#[cfg(test)]
mod test_lex {
    use super::*;

    fn test_lex(ss: Vec<&str>) {
        for s in ss.into_iter() {
            let tokens = SExpr::lex(s);
            assert_eq!(tokens, vec![s]);
        }
    }

    #[test]
    fn test_sexpr_lex_bool() {
        test_lex(vec!["true", "false"]);
    }

    #[test]
    fn test_sexpr_lex_number() {
        test_lex(vec!["1", "1.23"]);
    }

    #[test]
    fn test_sexpr_lex_str() {
        test_lex(vec!["\"hello world\"", "\"hello \\\" world\""]);
    }

    #[test]
    fn test_sexpr_lex_symbol() {
        test_lex(vec!["symbol"]);
    }

    #[test]
    fn test_sexpr_lex_complex() {
        let s = "(thing 2 (thing 1 1.23 'c' \"hello\\\" world\"))";
        let tokens = SExpr::lex(s);

        assert_eq!(
            tokens,
            vec![
                "(",
                "thing",
                "2",
                "(",
                "thing",
                "1",
                "1.23",
                "'c'",
                "\"hello\\\" world\"",
                ")",
                ")"
            ]
        );
    }
}
