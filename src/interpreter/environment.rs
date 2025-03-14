use std::collections::HashMap;

use crate::lex::Token;

use super::{error::RuntimeError, Value};

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    values: HashMap<&'a str, Option<Value<'a>>>,
    pub enclosing: Option<*mut Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Option<*mut Environment<'a>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: &'a str, value: Option<Value<'a>>) {
        if self.values.contains_key(name) {
            self.values.entry(name).and_modify(|v| *v = value);
        } else {
            self.values.insert(name, value);
        }
    }

    pub fn get(&self, name: Token<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        match self.values.get(name.lexeme) {
            Some(value) => match value {
                // TODO:
                Some(value) => Ok(value.clone()),
                None => Ok(Value::Nil),
            },
            None => match self.enclosing {
                Some(enclosing) => unsafe { return (*enclosing).get(name) },
                _ => Err(RuntimeError::UndefinedVariable { name }),
            },
        }
    }

    pub fn assign(&mut self, name: Token<'a>, value: Value<'a>) -> Result<(), RuntimeError<'a>> {
        match self.values.contains_key(name.lexeme) {
            true => {
                self.values
                    .entry(name.lexeme)
                    .and_modify(|v| *v = Some(value));
                Ok(())
            }
            false => match self.enclosing {
                Some(enclosing) => unsafe { (*enclosing).assign(name, value) },
                _ => Err(RuntimeError::UndefinedVariable { name }),
            },
        }
    }
}
