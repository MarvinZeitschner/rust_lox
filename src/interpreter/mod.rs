pub mod error;

use error::RuntimeError;

use crate::{ast::*, lex::TokenType};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Value {
    fn neg(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }
    fn not(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn add(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
            (Value::String(left), Value::String(right)) => {
                Ok(Value::String(format!("{}{}", left, right)))
            }
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn sub(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn div(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn mul(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn lt(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(*left < right)),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn gt(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(*left > right)),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn le(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(*left <= right)),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn ge(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(*left >= right)),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn eq(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(*left == right)),
            (Value::String(left), Value::String(right)) => Ok(Value::Boolean(*left == right)),
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(*left == right)),
            (Value::Nil, Value::Nil) => Ok(Value::Boolean(true)),
            _ => Ok(Value::Boolean(false)),
        }
    }

    fn ne(&self, other: Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(*left != right)),
            (Value::String(left), Value::String(right)) => Ok(Value::Boolean(*left != right)),
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(*left != right)),
            (Value::Nil, Value::Nil) => Ok(Value::Boolean(false)),
            _ => Ok(Value::Boolean(true)),
        }
    }
}

impl From<LiteralValue> for Value {
    fn from(literal: LiteralValue) -> Self {
        match literal {
            LiteralValue::F64(f) => Value::Number(f),
            LiteralValue::String(s) => Value::String(s),
            LiteralValue::Bool(b) => Value::Boolean(b),
            LiteralValue::Nil => Value::Nil,
        }
    }
}

#[derive(Default)]
pub struct Interpreter;

impl Interpreter {
    fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        expr.accept(self)
    }
}

impl Visitor for Interpreter {
    type Result = Result<Value, RuntimeError>;

    fn visit_literal(&mut self, node: &ExprLiteral) -> Self::Result {
        Ok(node.value.clone().into())
    }

    fn visit_grouping(&mut self, node: &ExprGrouping) -> Self::Result {
        node.value.accept(self)
    }

    fn visit_unary(&mut self, node: &ExprUnary) -> Self::Result {
        let right = node.value.accept(self)?;

        match node.operator.kind {
            TokenType::Minus => right.neg(),
            TokenType::Bang => right.not(),
            _ => Err(RuntimeError::InvalidOperator),
        }
    }

    fn visit_binary(&mut self, node: &ExprBinary) -> Self::Result {
        let left = node.left.accept(self)?;
        let right = node.right.accept(self)?;

        match node.operator.kind {
            TokenType::Minus => left.sub(right),
            TokenType::Slash => left.div(right),
            TokenType::Star => left.mul(right),
            TokenType::Plus => left.add(right),
            TokenType::Greater => left.gt(right),
            TokenType::Less => left.lt(right),
            TokenType::GreaterEqual => left.ge(right),
            TokenType::LessEqual => left.le(right),
            TokenType::EqualEqual => left.eq(right),
            TokenType::BangEqual => left.ne(right),
            _ => Ok(Value::Nil),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::lex::{Span, Token};

    use super::*;

    #[test]
    fn literal() {
        let mut interpreter = Interpreter;

        let expr = Expr::Literal(ExprLiteral::new(LiteralValue::F64(1.0)));
        let result = interpreter.evaluate(&expr).unwrap();

        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn grouping() {
        let mut interpreter = Interpreter;

        let expr = Expr::Grouping(ExprGrouping::new(Box::new(Expr::Literal(
            ExprLiteral::new(LiteralValue::F64(1.0)),
        ))));
        let result = interpreter.evaluate(&expr).unwrap();

        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn unary() {
        let mut interpreter = Interpreter;

        let span = Span { begin: 0, end: 1 };
        let token = Token::new(TokenType::Minus, "-", 1, span);
        let expr = Expr::Unary(ExprUnary::new(
            token,
            Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::F64(1.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();

        assert_eq!(result, Value::Number(-1.0));
    }

    #[test]
    fn error() {
        let mut interpreter = Interpreter;

        let span = Span { begin: 0, end: 1 };
        let token = Token::new(TokenType::Minus, "-", 1, span);
        let expr = Expr::Unary(ExprUnary::new(
            token,
            Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::String(
                "1".to_string(),
            )))),
        ));
        let result = interpreter.evaluate(&expr);

        assert_eq!(result, Err(RuntimeError::InvalidOperator));
    }
}
