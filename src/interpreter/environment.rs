use std::{cell::RefCell, rc::Rc};

use crate::ast::Local;

use super::Value;

/// The local variables of one scope at runtime. Variables are addressed by the slot the resolver
/// assigned them instead of by name, so a lookup is an index into a `Vec`. Globals live in the
/// interpreter instead, because they are looked up by name.
#[derive(Clone, Debug)]
pub struct Environment<'a> {
    values: Vec<Value<'a>>,
    pub enclosing: Option<Rc<RefCell<Environment<'a>>>>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Option<Rc<RefCell<Environment<'a>>>>) -> Self {
        Self::with_values(vec![], enclosing)
    }

    /// Creates an environment whose first slots are already filled, e.g. with the arguments of a
    /// function call.
    pub fn with_values(
        values: Vec<Value<'a>>,
        enclosing: Option<Rc<RefCell<Environment<'a>>>>,
    ) -> Self {
        Self { values, enclosing }
    }

    /// Adds the next variable of this scope. The resolver assigned slots in declaration order,
    /// so the variable lands on its slot.
    pub fn define(&mut self, value: Value<'a>) {
        self.values.push(value);
    }

    pub fn get_at(&self, local: Local) -> Value<'a> {
        match local.depth {
            0 => self.values[local.slot].clone(),
            depth => self.ancestor(depth).borrow().values[local.slot].clone(),
        }
    }

    pub fn assign_at(&mut self, local: Local, value: Value<'a>) {
        match local.depth {
            0 => self.values[local.slot] = value,
            depth => self.ancestor(depth).borrow_mut().values[local.slot] = value,
        }
    }

    /// Returns the environment `depth` hops up the chain. `depth` must be at least 1.
    fn ancestor(&self, depth: usize) -> Rc<RefCell<Environment<'a>>> {
        let mut environment = self.enclosing.clone().unwrap();
        for _ in 1..depth {
            let enclosing = environment.borrow().enclosing.clone().unwrap();
            environment = enclosing;
        }
        environment
    }
}
