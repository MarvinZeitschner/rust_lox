use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt,
    rc::{Rc, Weak},
};

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
    fn call(
        &self,
        interpreter: &mut Interpreter<'a>,
        arguments: VecDeque<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
    fn call_type(&self) -> CallType {
        CallType::Function
    }
    fn clone_as_class(&self) -> Option<Rc<LoxClass<'a>>> {
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
    pub closure: Weak<RefCell<Environment<'a>>>,
    pub is_initializer: bool,
}

impl<'a: 'b, 'b> LoxFunction<'a> {
    pub fn new(
        declaration: &'a StmtFunction<'b>,
        // closure: *mut Environment<'a>,
        closure: Weak<RefCell<Environment<'a>>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            declaration,
            closure,
            is_initializer,
        }
    }
    pub fn bind(&self, instance: LoxInstance<'a>) -> Self {
        let closure_env = self.closure.upgrade().map(|clos| Rc::downgrade(&clos));
        let environment = Rc::new(RefCell::new(Environment::new(closure_env)));

        environment.borrow_mut().define(
            "this",
            Some(Value::Instance(Rc::new(RefCell::new(instance)))),
        );

        Self {
            declaration: self.declaration,
            // closure: Box::into_raw(Box::new(environment)),
            closure: Rc::downgrade(&environment),
            is_initializer: self.is_initializer,
        }
    }

    pub fn bind_rc(&self, instance: Rc<RefCell<LoxInstance<'a>>>) -> Self {
        // let mut environment = Environment::new(Some(self.closure));
        // environment.define("this", Some(Value::Instance(instance)));

        let closure_env = self.closure.upgrade().map(|clos| Rc::downgrade(&clos));
        let environment = Rc::new(RefCell::new(Environment::new(closure_env)));

        environment
            .borrow_mut()
            .define("this", Some(Value::Instance(instance)));

        Self {
            declaration: self.declaration,
            closure: Rc::downgrade(&environment),
            is_initializer: self.is_initializer,
        }
    }
}

impl<'a> LoxCallable<'a> for LoxFunction<'a> {
    fn call(
        &self,
        interpreter: &mut Interpreter<'a>,
        mut arguments: VecDeque<Value<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        // let mut environment = Environment::new(Some(self.closure));
        let closure_env = self.closure.upgrade().map(|env| Rc::downgrade(&env));

        let environment = Rc::new(RefCell::new(Environment::new(closure_env)));

        for i in 0..self.declaration.params.len() {
            let lexeme = &self
                .declaration
                .params
                .get(i)
                .ok_or(CallableError::ParamNotFound)?
                .lexeme;

            let argument = arguments.pop_front().ok_or(CallableError::InternalError)?;

            environment.borrow_mut().define(lexeme, Some(argument));
        }

        let res = match interpreter.execute_block(&self.declaration.body, environment) {
            Ok(_) => Ok(Value::Nil),
            Err(err) => match err {
                RuntimeError::Return(value) => {
                    if self.is_initializer {
                        if let Some(closure) = self.closure.upgrade() {
                            return Ok(closure.borrow().get_at(0, "this"));
                        }
                        return Err(RuntimeError::EnvironmentCreationError);
                    }
                    Ok(value.value)
                }
                _ => Err(err),
            },
        };

        if self.is_initializer {
            if let Some(closure) = self.closure.upgrade() {
                return Ok(closure.borrow().get_at(0, "this"));
            }
            return Err(RuntimeError::EnvironmentCreationError);
        }

        res
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn to_string(&self) -> String {
        format!("<fn {}>", self.declaration.name.lexeme)
    }
}
