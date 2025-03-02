use std::collections::HashMap;

use crate::lex::Token;

use super::{error::RuntimeError, Value};

#[derive(Default)]
pub struct Environment<'a> {
    values: HashMap<&'a str, Option<Value>>,
}

impl<'a, 'b> Environment<'a> {
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

    pub fn get(&mut self, name: Token<'b>) -> Result<Value, RuntimeError<'b>> {
        match self.values.get(name.lexeme) {
            Some(value) => match value {
                Some(value) => Ok(value.clone()),
                None => Ok(Value::Nil),
            },
            None => Err(RuntimeError::UndefinedVariable { name }),
        }
    }
}
