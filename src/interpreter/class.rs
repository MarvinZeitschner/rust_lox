use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::lex::Token;

use super::{
    callable::{LoxCallable, LoxFunction},
    error::{ClassError, RuntimeError},
    value::Value,
};

#[derive(Debug, Clone)]
pub struct LoxClass<'a> {
    pub name: &'a str,
    pub methods: HashMap<&'a str, LoxFunction<'a>>,
}

impl<'a> LoxClass<'a> {
    pub fn new(name: &'a str, methods: HashMap<&'a str, LoxFunction<'a>>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction<'a>> {
        self.methods.get(name)
    }
}

impl<'a> LoxCallable<'a> for LoxClass<'a> {
    fn call(
        &self,
        _interpreter: &mut super::Interpreter<'a>,
        _arguments: std::collections::VecDeque<super::value::Value<'a>>,
    ) -> Result<super::value::Value<'a>, super::error::RuntimeError<'a>> {
        // TODO: Clone
        let instance = LoxInstance::new(self.clone());
        Ok(Value::Instance(Rc::new(RefCell::new(instance))))
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
        if let Some(value) = self.fields.get(name.lexeme) {
            // TODO: Clone
            return Ok(value.clone());
        }

        let method = self.class.find_method(name.lexeme);
        if let Some(method) = method {
            // TODO: Clone
            return Ok(Value::Callable(Rc::new(method.bind(self.clone()).clone())));
        }

        Err(RuntimeError::ClassError(ClassError::UndefinedProperty {
            token: name,
        }))
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
