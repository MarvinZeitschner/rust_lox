use std::{cell::RefCell, fmt, rc::Rc};

use rustc_hash::FxHashMap;

use crate::lex::Token;

use super::{
    callable::{LoxCallable, LoxFunction},
    error::{ClassError, RuntimeError},
    value::Value,
};

#[derive(Debug)]
pub struct LoxClass<'a> {
    pub name: &'a str,
    pub superclass: Option<Rc<LoxClass<'a>>>,
    pub methods: FxHashMap<&'a str, LoxFunction<'a>>,
}

impl<'a> LoxClass<'a> {
    pub fn new(
        name: &'a str,
        superclass: Option<Rc<LoxClass<'a>>>,
        methods: FxHashMap<&'a str, LoxFunction<'a>>,
    ) -> Self {
        Self {
            name,
            methods,
            superclass,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction<'a>> {
        if self.methods.contains_key(name) {
            return self.methods.get(name);
        }

        if let Some(superclass) = &self.superclass {
            return superclass.find_method(name);
        }

        None
    }
}

impl<'a> LoxCallable<'a> for LoxClass<'a> {
    fn call(
        self: Rc<Self>,
        interpreter: &mut super::Interpreter<'a>,
        arguments: Vec<super::value::Value<'a>>,
    ) -> Result<super::value::Value<'a>, super::error::RuntimeError<'a>> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(self.clone())));

        if let Some(initializer) = self.find_method("init") {
            initializer
                .bind(instance.clone())
                .invoke(interpreter, arguments)?;
        }

        Ok(Value::Instance(instance))
    }

    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            return initializer.arity();
        }
        0
    }

    fn to_string(&self) -> String {
        self.name.to_string()
    }

    fn as_class(self: Rc<Self>) -> Option<Rc<LoxClass<'a>>> {
        Some(self)
    }
}

pub struct LoxInstance<'a> {
    pub class: Rc<LoxClass<'a>>,
    pub fields: FxHashMap<&'a str, Value<'a>>,
}

impl<'a> LoxInstance<'a> {
    pub fn new(class: Rc<LoxClass<'a>>) -> Self {
        Self {
            class,
            fields: FxHashMap::default(),
        }
    }

    /// Looks up a field or method. Takes the instance's `Rc` so that methods are bound to this
    /// very instance and not to a copy of it.
    pub fn get(
        instance: &Rc<RefCell<LoxInstance<'a>>>,
        name: Token<'a>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        let this = instance.borrow();

        if let Some(value) = this.fields.get(name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(method) = this.class.find_method(name.lexeme) {
            return Ok(Value::Callable(Rc::new(method.bind(instance.clone()))));
        }

        Err(RuntimeError::ClassError(ClassError::UndefinedProperty {
            token: name,
        }))
    }

    pub fn set(&mut self, name: Token<'a>, value: Value<'a>) {
        self.fields.insert(name.lexeme, value);
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
