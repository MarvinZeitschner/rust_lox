use std::{collections::VecDeque, fmt};

use crate::ast::StmtFunction;

use super::{
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
pub struct LoxFunction<'a, 'b> {
    pub declaration: &'a StmtFunction<'b>,
}

impl<'a, 'b> LoxFunction<'a, 'b> {
    pub fn new(declaration: &'a StmtFunction<'b>) -> Self {
        Self { declaration }
    }
}

impl<'a, 'b> LoxCallable<'a> for LoxFunction<'a, 'b> {
    fn call(
        &self,
        interpreter: &mut Interpreter<'a>,
        mut arguments: VecDeque<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        let mut environment = Environment::new(Some(interpreter.get_mut_globals()));

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
