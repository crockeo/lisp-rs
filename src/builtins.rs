use std::collections::HashMap;

use crate::context::EvalContext;
use crate::error::{LispError, LispResult};
use crate::sexpr::SExpr;

pub type LispBuiltin = dyn Fn(&mut EvalContext, &Vec<&SExpr>) -> LispResult<SExpr> + Sync;

lazy_static! {
    static ref BUILTINS: HashMap<&'static str, Box<LispBuiltin>> = {
        let mut m: HashMap<&'static str, Box<LispBuiltin>> = HashMap::new();
        m.insert("+", Box::new(add_impl));
        m
    };
}

fn add_impl(_: &mut EvalContext, args: &Vec<&SExpr>) -> LispResult<SExpr> {
    let mut register = 0.0;
    for arg in args.into_iter() {
        if let SExpr::Num(f) = arg {
            register += f;
        } else {
            return Err(LispError::new(format!("attempted to add non-number '{}'", arg)));
        }
    }
    Ok(SExpr::Num(register))
}

pub fn eval_builtin(name: &str, ctx: &mut EvalContext, args: &Vec<&SExpr>) -> LispResult<SExpr> {
    let builtin = BUILTINS
        .get(name)
        .ok_or_else(|| LispError::new(format!("no such builtin '{}'", name)))?;
    builtin(ctx, args)
}
