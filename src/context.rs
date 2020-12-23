use crate::error::LispResult;
use crate::lisp_error;
use crate::sexpr::SExpr;

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

        lisp_error!("failed to get value with name '{}'", target_name);
    }

    /// set sets the value of the most locally-scoped
    pub fn set(&mut self, target_name: &str, new_value: SExpr) -> LispResult<()> {
        for (name, value) in self.values.iter_mut().rev() {
            if name.as_str() == target_name {
                *value = new_value;
                return Ok(());
            }
        }

        lisp_error!("failed to set value with name '{}'", target_name);
    }

    /// set_new creates a new binding the the name
    pub fn set_new(&mut self, name: &str, value: SExpr) {
        self.values.push((name.to_string(), value));
    }
}

