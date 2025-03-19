use std::{collections::HashMap, fmt};

use crate::lex::Token;

use super::{
    callable::LoxCallable,
    error::{ClassError, RuntimeError},
    value::Value,
};

#[derive(Debug, Clone)]
pub struct LoxClass<'a> {
    pub name: &'a str,
}

impl<'a> LoxClass<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name }
    }
}

impl<'a> LoxCallable<'a> for LoxClass<'a> {
    fn call(
        &self,
        interpreter: &mut super::Interpreter<'a>,
        arguments: std::collections::VecDeque<super::value::Value<'a>>,
    ) -> Result<super::value::Value<'a>, super::error::RuntimeError<'a>> {
        // TODO: Clone
        let instance = LoxInstance::new(self.clone());
        Ok(Value::Instance(instance))
    }

    fn arity(&self) -> usize {
        0
    }

    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Clone)]
pub struct LoxInstance<'a> {
    pub class: LoxClass<'a>,
    pub fields: HashMap<&'a str, Value<'a>>,
}

impl<'a> LoxInstance<'a> {
    pub fn new(class: LoxClass<'a>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: Token<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        // TODO: Clone
        self.fields
            .get(name.lexeme)
            .ok_or(RuntimeError::ClassError(ClassError::UndefinedProperty {
                token: name,
            }))
            .cloned()
    }

    pub fn set(&mut self, name: Token<'a>, value: Value<'a>) {
        if self.fields.contains_key(name.lexeme) {
            self.fields.entry(name.lexeme).and_modify(|v| *v = value);
        } else {
            self.fields.insert(name.lexeme, value);
        }
    }
}

impl<'a> fmt::Debug for LoxInstance<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> fmt::Display for LoxInstance<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
