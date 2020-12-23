use std::collections::HashMap;

use crate::context::EvalContext;
use crate::error::{LispError, LispResult};
use crate::lisp_error;
use crate::sexpr::SExpr;

pub type LispBuiltin = dyn Fn(&mut EvalContext, Vec<SExpr>) -> LispResult<SExpr> + Sync;

fn add_impl(_: &mut EvalContext, args: Vec<SExpr>) -> LispResult<SExpr> {
    let mut register = 0.0;
    for arg in args.into_iter() {
        if let SExpr::Num(f) = arg {
            register += f;
        } else {
            lisp_error!("attempted to add non-number '{}'", arg);
        }
    }
    Ok(SExpr::Num(register))
}

fn eval_impl(ctx: &mut EvalContext, args: Vec<SExpr>) -> LispResult<SExpr> {
    let mut ret = Ok(SExpr::List(Vec::new()));
    for arg in args.into_iter() {
        ret = Ok(match arg {
            SExpr::Symbol(name) => ctx.get(name.as_str())?.clone(),
            SExpr::List(exprs) => {
                if exprs.len() == 0 {
                    SExpr::List(exprs)
                } else {
                    let mut exprs = exprs.into_iter().map(|b| *b);
                    let head = exprs.next().unwrap(); // TODO: error handling
                    if let SExpr::Symbol(s) = head {
                        let args: Vec<SExpr> = exprs.collect();
                        if is_builtin(&s) {
                            return eval_builtin(&s, ctx, args);
                        } else if let Ok(SExpr::List(_)) = ctx.get(&s) {
                            todo!("run builtin funcall on args incl symbol name");
                        }

                        lisp_error!("failed to evaluate '{}', unknown builtin or function name", s);
                    }

                    lisp_error!("failed to evaluate list with head '{}'", head);
                }
            }
            x => x,
        })
    }

    ret
}

fn let_impl(ctx: &mut EvalContext, args: Vec<SExpr>) -> LispResult<SExpr> {
    todo!("implement let")
}

lazy_static! {
    static ref BUILTINS: HashMap<&'static str, Box<LispBuiltin>> = {
        let mut m: HashMap<&'static str, Box<LispBuiltin>> = HashMap::new();
        m.insert("+", Box::new(add_impl));
        m.insert("eval", Box::new(eval_impl));
        m.insert("let", Box::new(let_impl));
        m
    };
}

pub fn is_builtin(name: &str) -> bool {
    BUILTINS.contains_key(name)
}

pub fn eval_builtin(name: &str, ctx: &mut EvalContext, args: Vec<SExpr>) -> LispResult<SExpr> {
    let builtin = BUILTINS
        .get(name)
        .ok_or_else(|| LispError::new(format!("no such builtin '{}'", name)))?;
    builtin(ctx, args)
}
