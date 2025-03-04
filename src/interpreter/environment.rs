use std::collections::HashMap;

use crate::lex::Token;

use super::{error::RuntimeError, Value};

#[derive(Default, Clone)]
pub struct Environment<'a> {
    values: HashMap<&'a str, Option<Value>>,
    enclosing: Option<Box<Environment<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
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
                Some(value) => Ok(value.clone()),
                None => Ok(Value::Nil),
            },
            None => match &self.enclosing {
                Some(enclosing) => return enclosing.get(name),
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
                Some(enclosing) => enclosing.assign(name, value),
                _ => Err(RuntimeError::UndefinedVariable { name }),
            },
        }
    }
}

pub struct EnvironmentBuilder<'a> {
    encolsing: Option<Box<Environment<'a>>>,
}

impl<'a> EnvironmentBuilder<'a> {
    pub fn new() -> Self {
        Self { encolsing: None }
    }

    pub fn enclosing(mut self, enclosing: Environment<'a>) {
        self.encolsing = Some(Box::new(enclosing));
    }

    pub fn build(&self) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: self.encolsing.clone(),
        }
    }
}

impl<'a> Default for EnvironmentBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}
