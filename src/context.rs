use std::collections::HashMap;
use std::mem;

use crate::error::LispResult;
use crate::lisp_error;
use crate::sexpr::SExpr;

pub struct EvalContext {
    scopes: Vec<HashMap<String, SExpr>>,
}

impl EvalContext {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn get<'a>(&'a self, target_name: &str) -> LispResult<&'a SExpr> {
        for scope in self.scopes.iter().rev() {
            if let Some(sexpr) = scope.get(target_name) {
                return Ok(sexpr);
            }
        }

        lisp_error!("failed to retrieve symbol '{}'", target_name);
    }

    pub fn set<'a>(&'a mut self, target_name: &str, new_value: SExpr) -> LispResult<&'a SExpr> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sexpr) = scope.get_mut(target_name) {
                let _ = mem::replace(sexpr, new_value);
                return Ok(sexpr);
            }
        }

        lisp_error!("failed to set symbol '{}'", target_name);
    }

    pub fn push(&mut self, new_values: HashMap<String, SExpr>) -> LispResult<()> {
        self.scopes.push(new_values);
        Ok(())
    }

    pub fn pop(&mut self) -> LispResult<()> {
        if let None = self.scopes.pop() {
            lisp_error!("attempted to pop top-level EvalContext");
        }
        Ok(())
    }
}
