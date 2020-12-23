use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::context::EvalContext;
use crate::error::LispResult;
use crate::lisp_error;
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
        Ok(match self {
            SExpr::Symbol(name) => ctx.get(name.as_str())?.clone(),
            SExpr::List(exprs) => {
                if exprs.len() == 0 {
                    return Ok(SExpr::List(exprs));
                }

                let head = exprs[0].as_ref();
                if let SExpr::Symbol(s) = head {
                    let args: Vec<&SExpr> = exprs[1..].iter().map(|b| b.as_ref()).collect();
                    if let Ok(results) = eval_builtin(s, ctx, &args) {
                        return Ok(results);
                    }

                    if let Ok(SExpr::List(_)) = ctx.get(s) {
                        todo!("run builin funcall on args incl symbol name")
                    }

                    lisp_error!("failed to evaluate '{}', unknown builtin or function name", s);
                }

                lisp_error!("failed to evaluate list with head '{}'", head);
            }
            // for bool, number, and string, we map the value to itself
            x => x,
        })
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
