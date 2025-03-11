use crate::{
    ast::Expr,
    interpreter::{value::LoxCallable, Value},
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

impl LoxCallable for Clock {
    fn call(
        &self,
        _interpreter: &mut crate::interpreter::Interpreter,
        _arguments: Vec<&Expr>,
    ) -> crate::interpreter::Value {
        Value::Number(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        )
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn to_string(&self) -> String {
        String::from("<native fun: clock>")
    }
}
