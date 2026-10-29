use std::{cell::RefCell, collections::VecDeque, fmt, rc::Rc};

use crate::ast::StmtFunction;

use super::{
    class::{LoxClass, LoxInstance},
    environment::Environment,
    error::{CallableError, RuntimeError},
    value::Value,
    Interpreter,
};

pub enum CallType {
    Class,
    Function,
}

pub trait LoxCallable<'a>: 'a {
    /// Takes `Rc<Self>` so that a class can hand out a reference to itself to its instances.
    fn call(
        self: Rc<Self>,
        interpreter: &mut Interpreter<'a>,
        arguments: VecDeque<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
    fn call_type(&self) -> CallType {
        CallType::Function
    }
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
    pub closure: Rc<RefCell<Environment<'a>>>,
    pub is_initializer: bool,
}

impl<'a> LoxFunction<'a> {
    pub fn new(
        declaration: &'a StmtFunction<'a>,
        closure: Rc<RefCell<Environment<'a>>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            declaration,
            closure,
            is_initializer,
        }
    }
    pub fn bind(&self, instance: Rc<RefCell<LoxInstance<'a>>>) -> Self {
        let mut environment = Environment::new(Some(self.closure.clone()));
        environment.define("this", Value::Instance(instance));
        Self {
            declaration: self.declaration,
            closure: Rc::new(RefCell::new(environment)),
            is_initializer: self.is_initializer,
        }
    }

    pub fn invoke(
        &self,
        interpreter: &mut Interpreter<'a>,
        mut arguments: VecDeque<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        let mut environment = Environment::new(Some(self.closure.clone()));

        for i in 0..self.declaration.params.len() {
            let lexeme = &self
                .declaration
                .params
                .get(i)
                .ok_or(CallableError::ParamNotFound)?
                .lexeme;

            let argument = arguments.pop_front().ok_or(CallableError::InternalError)?;

            environment.define(lexeme, argument);
        }

        match interpreter.execute_block(&self.declaration.body, environment) {
            // An initializer always returns `this`, even after an early `return;`
            Ok(()) | Err(RuntimeError::Return(_)) if self.is_initializer => {
                Ok(self.closure.borrow().get_at(0, "this"))
            }
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
        arguments: VecDeque<Value<'a>>,
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
