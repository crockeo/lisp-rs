use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::Write;
use std::str::FromStr;

// TODO: decomp all of these components into their own homes; SExpr into their place, EvalContext
// into its own place (maybe along with the eval function on the SExpr struct), etc.

pub struct LispError {
    message: String,
    // TODO: this is currently unused, but should be used Soon™ so that you can figure out where
    // errors are.
    _row: usize,
    _col: usize,
}

impl LispError {
    fn new<S: AsRef<str>>(message: S) -> LispError {
        LispError {
            message: message.as_ref().to_string(),
            _row: 0,
            _col: 0,
        }
    }
}

pub type LispResult<T> = Result<T, LispError>;

pub struct EvalContext {
    values: Vec<(String, SExpr)>,
}

impl EvalContext {
    pub fn new() -> EvalContext {
        EvalContext { values: Vec::new() }
    }

    /// get retrieves a value from the EvalContext if it exists, otherwise None.
    pub fn get(&mut self, target_name: &str) -> LispResult<&SExpr> {
        for (name, value) in self.values.iter().rev() {
            if name.as_str() == target_name {
                return Ok(value);
            }
        }

        Err(LispError::new(format!(
            "failed to get value with name '{}'",
            target_name
        )))
    }

    /// set sets the value of the most locally-scoped
    pub fn set(&mut self, target_name: &str, new_value: SExpr) -> LispResult<()> {
        for (name, value) in self.values.iter_mut().rev() {
            if name.as_str() == target_name {
                *value = new_value;
                return Ok(());
            }
        }

        Err(LispError::new(format!(
            "failed to set value with name '{}'",
            target_name,
        )))
    }

    /// set_new creates a new binding the the name
    pub fn set_new(&mut self, name: &str, value: SExpr) {
        self.values.push((name.to_string(), value));
    }
}

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
                    if let Some(results) = eval_builtin(s, args) {
                        return Ok(results);
                    }

                    if let Ok(SExpr::List(_)) = ctx.get(s) {
                        todo!("run builin funcall on args incl symbol name")
                    }

                    return Err(LispError::new(format!(
                        "failed to evaluate '{}', unknown builtin or function name",
                        s,
                    )));
                }

                return Err(LispError::new(format!(
                    "failed to evaluate list with head '{}'",
                    head
                )));
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

pub fn eval_builtin<'a, I: IntoIterator<Item = &'a SExpr>>(name: &str, args: I) -> Option<SExpr> {
    if name == "+" {
        let mut register = 0.0;
        for arg in args.into_iter() {
            if let SExpr::Num(f) = arg {
                register += f;
            } else {
                return None;
            }
        }
        return Some(SExpr::Num(register));
    }

    return None;
}

fn main() -> io::Result<()> {
    let mut eval_context = EvalContext::new();
    loop {
        let line = read_line("> ")?;
        let tokens = SExpr::lex(line.as_str());
        let mut tokens_iter = tokens.into_iter();
        while let Some(expr) = SExpr::parse(&mut tokens_iter) {
            match expr.eval(&mut eval_context) {
                Err(e) => {
                    println!("encountered error: {}", e.message);
                },
                Ok(result) => println!("{}", result),
            }
        }

        let result = read_line("> ")?
            .as_str()
            .parse::<SExpr>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
            .eval(&mut eval_context);

        println!(
            "{}",
            result.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.message))?,
        );
    }
}

/// read_line reads a single line of input from the user and returns it as an owned string WITHOUT
/// newline.
fn read_line(prompt: &str) -> io::Result<String> {
    let mut line = String::new();
    io::stdout().write(prompt.as_bytes())?;
    io::stdout().flush()?;
    io::stdin().read_line(&mut line)?;
    while let Some(c) = line.chars().next_back() {
        if c == '\n' || c == '\r' {
            line.pop();
        } else {
            break;
        }
    }

    Ok(line)
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
