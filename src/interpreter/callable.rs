use std::{cell::RefCell, collections::VecDeque, fmt, rc::Rc};

use crate::ast::StmtFunction;

use super::{
    class::LoxInstance,
    environment::Environment,
    error::{CallableError, RuntimeError},
    value::Value,
    Interpreter,
};

pub trait LoxCallable<'a>: 'a {
    fn call(
        &self,
        interpreter: &mut Interpreter<'a>,
        arguments: VecDeque<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
}

impl<'a> fmt::Debug for dyn LoxCallable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct LoxFunction<'a> {
    pub declaration: &'a StmtFunction<'a>,
    pub closure: *mut Environment<'a>,
}

impl<'a: 'b, 'b> LoxFunction<'a> {
    pub fn new(declaration: &'a StmtFunction<'b>, closure: *mut Environment<'a>) -> Self {
        Self {
            declaration,
            closure,
        }
    }
    pub fn bind(&self, instance: LoxInstance<'a>) -> Self {
        let mut environment = Environment::new(Some(self.closure));
        environment.define(
            "this",
            Some(Value::Instance(Rc::new(RefCell::new(instance)))),
        );

        Self {
            declaration: self.declaration,
            closure: Box::into_raw(Box::new(environment)),
        }
    }
}

impl<'a: 'b, 'b> LoxCallable<'a> for LoxFunction<'a> {
    fn call(
        &self,
        interpreter: &mut Interpreter<'a>,
        mut arguments: VecDeque<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        let mut environment = Environment::new(Some(self.closure));

        for i in 0..self.declaration.params.len() {
            let lexeme = &self
                .declaration
                .params
                .get(i)
                .ok_or(CallableError::ParamNotFound)?
                .lexeme;

            let argument = arguments.pop_front().ok_or(CallableError::InternalError)?;

            environment.define(lexeme, Some(argument));
        }

        match interpreter.execute_block(&self.declaration.body, environment) {
            Ok(_) => Ok(Value::Nil),
            Err(err) => match err {
                // Safe to unwrap since there is already a check in the interpreter for this
                RuntimeError::Return(value) => Ok(value.value.unwrap()),
                _ => Err(err),
            },
        }
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn to_string(&self) -> String {
        format!("<fn {} >", self.declaration.name.lexeme)
    }
}
