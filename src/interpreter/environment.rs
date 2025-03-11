use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::lex::Token;

use super::{error::RuntimeError, Value};

#[derive(Clone)]
pub struct Environment<'a> {
    values: HashMap<&'a str, Option<Value>>,
    // pub enclosing: Option<Box<Environment<'a>>>,
    pub enclosing: Option<Rc<RefCell<Environment<'a>>>>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Option<Rc<RefCell<Environment<'a>>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: &'a str, value: Option<Value>) {
        if self.values.contains_key(name) {
            self.values.entry(name).and_modify(|v| *v = value);
        } else {
            self.values.insert(name, value);
        }
    }

    pub fn get(&self, name: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match self.values.get(name.lexeme) {
            Some(value) => match value {
                // TODO:
                Some(value) => Ok(value.clone()),
                None => Ok(Value::Nil),
            },
            None => match &self.enclosing {
                Some(enclosing) => return enclosing.borrow().get(name),
                _ => Err(RuntimeError::UndefinedVariable { name }),
            },
        }
    }

    pub fn assign(&mut self, name: Token<'a>, value: Value) -> Result<(), RuntimeError<'a>> {
        match self.values.contains_key(name.lexeme) {
            true => {
                self.values
                    .entry(name.lexeme)
                    .and_modify(|v| *v = Some(value));
                Ok(())
            }
            false => match &mut self.enclosing {
                Some(enclosing) => enclosing.borrow_mut().assign(name, value),
                _ => Err(RuntimeError::UndefinedVariable { name }),
            },
        }
    }
}
