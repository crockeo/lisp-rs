use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::context::EvalContext;
use crate::error::LispResult;
use crate::builtins::eval_builtin;

#[derive(Clone)]
pub enum SExpr {
    Bool(bool),
    Num(f64),
    Str(String),
    Symbol(String),
    List(Vec<Box<SExpr>>),
}

impl SExpr {
    /// lex maps a string into a series of tokens that can then be parsed into an S Expression.
    pub fn lex<'a>(s: &'a str) -> Vec<&'a str> {
        let mut tokens = Vec::new();

        // TODO: clean this up Some Time™
        let mut start_info: Option<(usize, char)> = None;
        let mut escaped = false;
        for (i, c) in s.chars().enumerate() {
            if c == '(' {
                tokens.push("(");
            } else if c == ')' {
                if let Some((sp, _)) = start_info {
                    tokens.push(&s[sp..i]);
                    start_info = None;
                }
                tokens.push(")");
            } else if !c.is_whitespace() && start_info == None {
                start_info = Some((i, c));
            } else if let Some((si, sc)) = start_info {
                if sc == '"' {
                    if c == '\\' {
                        escaped = true;
                    } else if c == '"' && !escaped {
                        tokens.push(&s[si..i + 1]);
                        start_info = None;
                        escaped = false;
                    }
                } else if c.is_whitespace() {
                    tokens.push(&s[si..i]);
                    start_info = None;
                }
            }
        }

        if let Some((sp, _)) = start_info {
            tokens.push(&s[sp..]);
        }

        tokens
    }

    /// parse transforms lexed input into a structured S expression.
    pub fn parse<'a, I: Iterator<Item = &'a str>>(mut iter: I) -> Option<Self> {
        // TODO: translate this and the other vestigial uses of Option over to LispResult
        let head = iter.next()?;
        let head_chars: Vec<char> = head.chars().collect();

        Some(if head == "true" {
            SExpr::Bool(true)
        } else if head == "false" {
            SExpr::Bool(false)
        } else if head_chars[0] == '"' && head_chars[head_chars.len() - 1] == '"' {
            SExpr::Str(head_chars[1..head_chars.len() - 1].iter().collect())
        } else if let Some(f) = head.parse::<f64>().ok() {
            SExpr::Num(f)
        } else if head == "(" {
            // TODO: clean up list parsing
            let mut raw_children = Vec::new();
            let mut depth = 1;
            while let Some(raw_child) = iter.next() {
                if raw_child == "(" {
                    depth += 1;
                } else if raw_child == ")" {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                raw_children.push(raw_child);
            }

            let mut raw_children_iter = raw_children.into_iter();
            let mut children = Vec::new();
            while let Some(child) = SExpr::parse(&mut raw_children_iter) {
                children.push(Box::new(child));
            }
            SExpr::List(children)
        } else {
            SExpr::Symbol(head.to_string())
        })
    }

    pub fn eval(self, ctx: &mut EvalContext) -> LispResult<Self> {
        eval_builtin("eval", ctx, vec![self])
    }
}

impl Display for SExpr {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            SExpr::Bool(b) => write!(fmt, "{}", b),
            SExpr::Num(f) => write!(fmt, "{}", f),
            SExpr::Str(s) => write!(fmt, "\"{}\"", s),
            SExpr::Symbol(s) => write!(fmt, "{}", s),
            SExpr::List(exprs) => {
                write!(fmt, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    write!(fmt, "{}", expr)?;
                    if i < exprs.len() - 1 {
                        write!(fmt, " ")?;
                    }
                }
                write!(fmt, ")")
            }
        }
    }
}

impl FromStr for SExpr {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SExpr::parse(SExpr::lex(s).into_iter()).ok_or("failed to parse SExpr")
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
