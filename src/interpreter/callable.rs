use std::{cell::RefCell, fmt, rc::Rc};

use crate::ast::{Local, StmtFunction};

use super::{
    class::{LoxClass, LoxInstance},
    environment::Environment,
    error::RuntimeError,
    value::Value,
    Interpreter,
};

pub trait LoxCallable<'a>: 'a {
    /// Takes `Rc<Self>` so that a class can hand out a reference to itself to its instances.
    fn call(
        self: Rc<Self>,
        interpreter: &mut Interpreter<'a>,
        arguments: Vec<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
    fn as_class(self: Rc<Self>) -> Option<Rc<LoxClass<'a>>> {
        None
    }
}

impl<'a> fmt::Debug for dyn LoxCallable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct LoxFunction<'a> {
    pub declaration: &'a StmtFunction<'a>,
    /// The environment the function was defined in. `None` for functions defined at the top
    /// level, whose surrounding variables are the globals.
    pub closure: Option<Rc<RefCell<Environment<'a>>>>,
    pub is_initializer: bool,
}

impl<'a> LoxFunction<'a> {
    pub fn new(
        declaration: &'a StmtFunction<'a>,
        closure: Option<Rc<RefCell<Environment<'a>>>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            declaration,
            closure,
            is_initializer,
        }
    }
    /// Returns this method with `this` bound to `instance`, in slot 0 of a new environment.
    pub fn bind(&self, instance: Rc<RefCell<LoxInstance<'a>>>) -> Self {
        let environment =
            Environment::with_values(vec![Value::Instance(instance)], self.closure.clone());
        Self {
            declaration: self.declaration,
            closure: Some(Rc::new(RefCell::new(environment))),
            is_initializer: self.is_initializer,
        }
    }

    pub fn invoke(
        &self,
        interpreter: &mut Interpreter<'a>,
        arguments: Vec<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        // The parameters are the first variables of the function's scope, so the arguments
        // already are the first slots of its environment. `visit_call` checked the arity.
        let environment = Environment::with_values(arguments, self.closure.clone());

        match interpreter.execute_block(&self.declaration.body, environment) {
            // An initializer always returns `this`, even after an early `return;`
            Ok(()) | Err(RuntimeError::Return(_)) if self.is_initializer => Ok(self
                .closure
                .as_ref()
                .expect("initializers are always bound to an instance")
                .borrow()
                .get_at(Local { depth: 0, slot: 0 })),
            Ok(()) => Ok(Value::Nil),
            Err(RuntimeError::Return(value)) => Ok(value.value),
            Err(err) => Err(err),
        }
    }
}

impl<'a> LoxCallable<'a> for LoxFunction<'a> {
    fn call(
        self: Rc<Self>,
        interpreter: &mut Interpreter<'a>,
        arguments: Vec<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        self.invoke(interpreter, arguments)
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn to_string(&self) -> String {
        format!("<fn {}>", self.declaration.name.lexeme)
    }
}
