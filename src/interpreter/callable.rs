use std::{collections::VecDeque, fmt};

use crate::ast::StmtFunction;

use super::{
    environment::Environment,
    error::{CallableError, RuntimeError},
    value::Value,
    Interpreter,
};

pub trait LoxCallable<'a, R = Value<'a>> {
    fn call(
        &self,
        interpreter: &mut Interpreter<'a>,
        arguments: VecDeque<Value<'a>>,
    ) -> Result<R, RuntimeError<'a>>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
    fn clone_box(&self) -> Box<dyn LoxCallable<'a>>;
}

impl<'a> Clone for Box<dyn LoxCallable<'a>> {
    fn clone(&self) -> Box<dyn LoxCallable<'a>> {
        self.clone_box()
    }
}

impl<'a> fmt::Debug for Box<dyn LoxCallable<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct LoxFunction<'a> {
    pub declaration: StmtFunction<'a>,
}

impl<'a> LoxFunction<'a> {
    pub fn new(declaration: StmtFunction<'a>) -> Self {
        Self { declaration }
    }
}

impl<'a> LoxCallable<'a, ()> for LoxFunction<'a> {
    fn call(
        &self,
        interpreter: &mut Interpreter<'a>,
        mut arguments: VecDeque<Value<'a>>,
    ) -> Result<(), RuntimeError<'a>> {
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

        interpreter.execute_block(&self.declaration.body, environment)?;

        Ok(())
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn to_string(&self) -> String {
        format!("<fn {} >", self.declaration.name.lexeme)
    }

    fn clone_box(&self) -> Box<dyn LoxCallable<'a>> {
        todo!()
    }
}
