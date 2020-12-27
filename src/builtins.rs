use std::collections::HashMap;

use crate::context::EvalContext;
use crate::error::{LispError, LispResult};
use crate::lisp_error;
use crate::sexpr::SExpr;

pub type LispBuiltin = dyn Fn(&mut EvalContext, Vec<SExpr>) -> LispResult<SExpr> + Sync;

fn add_impl(ctx: &mut EvalContext, args: Vec<SExpr>) -> LispResult<SExpr> {
    let mut register = 0.0;
    for arg in args.into_iter() {
        let mut arg = &arg;
        if let SExpr::Symbol(s) = arg {
            arg = ctx.get(s)?;
        }

        if let SExpr::Num(f) = arg {
            register += f;
        } else {
            lisp_error!("attempted to add non-number '{}'", arg);
        }
    }
    Ok(SExpr::Num(register))
}

fn def_impl(ctx: &mut EvalContext, args: Vec<SExpr>) -> LispResult<SExpr> {
    if args.len() != 2 {
        lisp_error!("def must be used with exactly 2 args, not {}", args.len());
    }

    let mut iter = args.into_iter();
    let symbol = iter.next().unwrap();
    let value = iter.next().unwrap();

    if let SExpr::Symbol(s) = symbol {
        let mut new_values = HashMap::new();
        new_values.insert(s, eval_impl(ctx, vec![value.clone()])?);
        ctx.push(new_values)?;
        Ok(value)
    } else {
        lisp_error!("cannot define place with non-symbol name");
    }
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

                        lisp_error!(
                            "failed to evaluate '{}', unknown builtin or function name",
                            s
                        );
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
    let mut iter = args.into_iter();
    let bindings = iter.next();
    if let Some(SExpr::List(bindings)) = bindings {
        let mut new_values: HashMap<String, SExpr> = HashMap::new();
        for binding in bindings.into_iter() {
            if let SExpr::List(binding) = *binding {
                if binding.len() != 2 {
                    lisp_error!("invalid let binding");
                }

                let mut iter = binding.into_iter();
                let name = *iter.next().unwrap();
                let value = *iter.next().unwrap();

                if let SExpr::Symbol(s) = name {
                    new_values.insert(s, eval_impl(ctx, vec![value])?);
                } else {
                    lisp_error!("let binding must contain ");
                }
            } else {
                lisp_error!("let binding must be list");
            }
        }
        ctx.push(new_values)?;
    } else {
        lisp_error!("let must contain bindings list");
    }

    let mut result = None;
    for body in iter {
        result = Some(eval_impl(ctx, vec![body]));
    }

    ctx.pop()?;

    match result {
        None => Ok(SExpr::List(vec![])),
        Some(x) => x,
    }
}

lazy_static! {
    static ref BUILTINS: HashMap<&'static str, Box<LispBuiltin>> = {
        let mut m: HashMap<&'static str, Box<LispBuiltin>> = HashMap::new();
        m.insert("+", Box::new(add_impl));
        m.insert("def", Box::new(def_impl));
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
