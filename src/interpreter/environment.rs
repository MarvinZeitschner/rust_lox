use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::lex::Token;

use super::{error::RuntimeError, Value};

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    values: HashMap<&'a str, Value<'a>>,
    pub enclosing: Option<Rc<RefCell<Environment<'a>>>>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Option<Rc<RefCell<Environment<'a>>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: &'a str, value: Value<'a>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: Token<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        match self.values.get(name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get(name),
                _ => Err(RuntimeError::UndefinedVariable { name }),
            },
        }
    }

    pub fn get_at(&self, distance: usize, name: &'a str) -> Value<'a> {
        // The resolver already found the variable at this depth, so it is guaranteed to exist
        let get = |env: &Environment<'a>| {
            env.values
                .get(name)
                .expect("resolver and environments disagree about a variable")
                .clone()
        };

        match distance {
            0 => get(self),
            _ => get(&self.ancestor(distance).borrow()),
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: Token<'a>, value: Value<'a>) {
        match distance {
            0 => self.define(name.lexeme, value),
            _ => self
                .ancestor(distance)
                .borrow_mut()
                .define(name.lexeme, value),
        }
    }

    /// Returns the environment `distance` hops up the chain. `distance` must be at least 1.
    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment<'a>>> {
        let mut environment = self.enclosing.clone().unwrap();
        for _ in 1..distance {
            let enclosing = environment.borrow().enclosing.clone().unwrap();
            environment = enclosing;
        }
        environment
    }

    pub fn assign(&mut self, name: Token<'a>, value: Value<'a>) -> Result<(), RuntimeError<'a>> {
        match self.values.get_mut(name.lexeme) {
            Some(v) => {
                *v = value;
                Ok(())
            }
            None => match &self.enclosing {
                Some(enclosing) => enclosing.borrow_mut().assign(name, value),
                _ => Err(RuntimeError::UndefinedVariable { name }),
            },
        }
    }
}
