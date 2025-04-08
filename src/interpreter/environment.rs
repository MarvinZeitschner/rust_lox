use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::lex::Token;

use super::{error::RuntimeError, Value};

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    values: HashMap<&'a str, Option<Value<'a>>>,
    pub enclosing: Option<Weak<RefCell<Environment<'a>>>>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Option<Weak<RefCell<Environment<'a>>>>) -> Self {
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
                Some(value) => Ok(value.clone()),
                None => Ok(Value::Nil),
            },
            None => match &self.enclosing {
                Some(enclosing) => match enclosing.upgrade() {
                    Some(enc) => enc.borrow().get(name),
                    None => Err(RuntimeError::UndefinedVariable { name }),
                },
                None => Err(RuntimeError::UndefinedVariable { name }),
            },
        }
    }

    pub fn get_at(&self, distance: usize, name: &'a str) -> Value<'a> {
        let env = self.ancestor(distance);

        match env {
            Some(env) => env
                .borrow()
                .values
                .get(name)
                .unwrap_or_else(|| panic!("Variable '{}' not found at distance {}", name, distance))
                .clone()
                .unwrap_or_else(|| panic!("Variable '{}' is None at distance {}", name, distance)),

            None => self
                .values
                .get(name)
                .unwrap_or_else(|| panic!("Variable '{}' not found at distance {}", name, distance))
                .clone()
                .unwrap_or_else(|| panic!("Variable '{}' is None at distance {}", name, distance)),
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: Token<'a>, value: Value<'a>) {
        let env = self.ancestor(distance);

        match env {
            Some(env) => env.borrow_mut().define(name.lexeme, Some(value)),
            None => self.define(name.lexeme, Some(value)),
        }
    }

    fn ancestor(&self, distance: usize) -> Option<Rc<RefCell<Environment<'a>>>> {
        if distance == 0 {
            return None;
        }

        let mut environment = match &self.enclosing {
            Some(enc) => enc.upgrade().unwrap_or_else(|| {
                panic!(
                    "Environment was dropped (looking for distance {})",
                    distance
                )
            }),
            None => panic!(
                "Ancestor lookup failed: no enclosing environment (looking for distance {})",
                distance
            ),
        };

        for i in 1..distance {
            let next_env = match &environment.borrow().enclosing {
                Some(enc) => enc.upgrade().unwrap_or_else(|| panic!("Environment was dropped (looking for distance {}, current: {})",
                     distance, i)),
                None => panic!(
                    "Ancestor lookup failed: environment chain too short (looking for distance {}, current: {})",
                     distance, i
                ),
            };
            environment = next_env;
        }

        Some(environment)
    }

    pub fn assign(&mut self, name: Token<'a>, value: Value<'a>) -> Result<(), RuntimeError<'a>> {
        match self.values.contains_key(name.lexeme) {
            true => {
                self.values
                    .entry(name.lexeme)
                    .and_modify(|v| *v = Some(value));
                Ok(())
            }
            false => match &self.enclosing {
                Some(enclosing) => match enclosing.upgrade() {
                    Some(enc) => enc.borrow_mut().assign(name, value),
                    None => Err(RuntimeError::UndefinedVariable { name }),
                },
                _ => Err(RuntimeError::UndefinedVariable { name }),
            },
        }
    }
}
