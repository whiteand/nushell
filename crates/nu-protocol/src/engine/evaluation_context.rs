use super::EngineState;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{Example, ShellError, Signature, Value, VarId};

#[derive(Clone)]
pub struct EvaluationContext {
    pub engine_state: Rc<RefCell<EngineState>>,
    pub stack: Stack,
}

impl EvaluationContext {
    pub fn get_var(&self, var_id: VarId) -> Result<Value, ShellError> {
        self.stack.get_var(var_id)
    }

    pub fn enter_scope(&self) -> EvaluationContext {
        Self {
            engine_state: self.engine_state.clone(),
            stack: self.stack.clone().enter_scope(),
        }
    }

    pub fn add_var(&self, var_id: VarId, value: Value) {
        // We need to make values concreate before we assign them to variables, as stream values
        // will drain and remain drained.
        //
        // TODO: find a good home for converting a stream->list when storing into a variable
        // TODO: add ctrl-c support when setting a var

        let value = match value {
            Value::Stream { stream, span } => Value::List {
                vals: stream.collect(),
                span,
            },
            x => x,
        };

        self.stack.add_var(var_id, value);
    }

    pub fn add_env_var(&self, var: String, value: String) {
        self.stack.add_env_var(var, value);
    }

    pub fn print_stack(&self) {
        self.stack.print_stack();
    }

    pub fn get_signatures(&self) -> Vec<Signature> {
        self.engine_state.borrow().get_signatures()
    }

    pub fn get_signatures_with_examples(&self) -> Vec<(Signature, Vec<Example>)> {
        self.engine_state.borrow().get_signatures_with_examples()
    }
}

#[derive(Debug)]
pub struct StackFrame {
    pub vars: HashMap<VarId, Value>,
    pub env_vars: HashMap<String, String>,
    pub parent: Option<Stack>,
}

#[derive(Clone, Debug)]
pub struct Stack(Rc<RefCell<StackFrame>>);

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    pub fn new() -> Stack {
        Stack(Rc::new(RefCell::new(StackFrame {
            vars: HashMap::new(),
            env_vars: HashMap::new(),
            parent: None,
        })))
    }
    pub fn get_var(&self, var_id: VarId) -> Result<Value, ShellError> {
        let this = self.0.borrow();
        match this.vars.get(&var_id) {
            Some(v) => Ok(v.clone()),
            _ => {
                if let Some(parent) = &this.parent {
                    parent.get_var(var_id)
                } else {
                    Err(ShellError::InternalError("variable not found".into()))
                }
            }
        }
    }

    pub fn add_var(&self, var_id: VarId, value: Value) {
        let mut this = self.0.borrow_mut();
        this.vars.insert(var_id, value);
    }

    pub fn add_env_var(&self, var: String, value: String) {
        let mut this = self.0.borrow_mut();
        this.env_vars.insert(var, value);
    }

    pub fn enter_scope(self) -> Stack {
        Stack(Rc::new(RefCell::new(StackFrame {
            vars: HashMap::new(),
            env_vars: HashMap::new(),
            parent: Some(self),
        })))
    }

    pub fn get_env_vars(&self) -> HashMap<String, String> {
        self.0.borrow().env_vars.clone()
    }

    pub fn get_env_var(&self, name: &str) -> Option<String> {
        self.0.borrow().env_vars.get(name).cloned()
    }

    pub fn print_stack(&self) {
        println!("===frame===");
        println!("vars:");
        for (var, val) in &self.0.borrow().vars {
            println!("  {}: {:?}", var, val);
        }
        println!("env vars:");
        for (var, val) in &self.0.borrow().env_vars {
            println!("  {}: {:?}", var, val);
        }
        if let Some(parent) = &self.0.borrow().parent {
            parent.print_stack()
        }
    }
}
