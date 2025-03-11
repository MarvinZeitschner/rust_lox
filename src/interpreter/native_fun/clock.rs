use crate::{
    ast::Expr,
    interpreter::{error::RuntimeError, value::LoxCallable, Interpreter, Value},
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Clock {
    arity: usize,
}

impl Clock {
    pub fn new() -> Self {
        Self { arity: 0 }
    }
}

impl<'a> LoxCallable<'a> for Clock {
    fn call(
        &self,
        _interpreter: &mut Interpreter<'a>,
        _arguments: Vec<&Expr<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        Ok(Value::Number(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        ))
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn to_string(&self) -> String {
        String::from("<native fun: clock>")
    }

    fn clone_box(&self) -> Box<dyn LoxCallable<'a>> {
        Box::new(*self)
    }
}
