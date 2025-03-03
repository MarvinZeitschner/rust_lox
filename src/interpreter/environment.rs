use std::collections::HashMap;

use crate::lex::Token;

use super::{error::RuntimeError, Value};

#[derive(Default)]
pub struct Environment<'a> {
    values: HashMap<&'a str, Option<Value>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &'a str, value: Option<Value>) {
        if self.values.contains_key(name) {
            self.values.entry(name).and_modify(|v| *v = value);
        } else {
            self.values.insert(name, value);
        }
    }

    pub fn get(&mut self, name: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match self.values.get(name.lexeme) {
            Some(value) => match value {
                Some(value) => Ok(value.clone()),
                None => Ok(Value::Nil),
            },
            None => Err(RuntimeError::UndefinedVariable { name }),
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
            false => Err(RuntimeError::UndefinedVariable { name }),
        }
    }
}
