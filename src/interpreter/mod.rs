pub mod error;

use error::RuntimeError;

use crate::{
    ast::*,
    lex::{Token, TokenType},
};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl<'a> Value {
    fn neg(self, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match self {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }
    fn not(self, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match self {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn add(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
            (Value::String(left), Value::String(right)) => {
                Ok(Value::String(format!("{}{}", left, right)))
            }
            _ => Err(RuntimeError::NumberOrStringOperands { operator }),
        }
    }

    fn sub(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn div(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn mul(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn lt(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left < right)),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn gt(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left > right)),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn le(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left <= right)),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn ge(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left >= right)),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn eq(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left == right)),
            (Value::String(left), Value::String(right)) => Ok(Value::Boolean(left == right)),
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(left == right)),
            (Value::Nil, Value::Nil) => Ok(Value::Boolean(true)),
            _ => Ok(Value::Boolean(false)),
        }
    }

    fn ne(self, other: Value, operator: Token<'a>) -> Result<Value, RuntimeError<'a>> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left != right)),
            (Value::String(left), Value::String(right)) => Ok(Value::Boolean(left != right)),
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(left != right)),
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

impl<'a> Interpreter {
    fn evaluate(&mut self, expr: &Expr<'a>) -> Result<Value, RuntimeError<'a>> {
        expr.accept(self)
    }
}

impl<'a> Visitor<'a> for Interpreter {
    type Output = Result<Value, RuntimeError<'a>>;

    fn visit_literal(&mut self, node: &ExprLiteral) -> Self::Output {
        Ok(node.value.clone().into())
    }

    fn visit_grouping(&mut self, node: &ExprGrouping<'a>) -> Self::Output {
        node.value.accept(self)
    }

    fn visit_unary(&mut self, node: &ExprUnary<'a>) -> Self::Output {
        let operator = node.operator;
        let right = node.value.accept(self)?;

        match node.operator.kind {
            TokenType::Minus => right.neg(operator),
            TokenType::Bang => right.not(operator),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn visit_binary(&mut self, node: &ExprBinary<'a>) -> Self::Output {
        let operator = node.operator;
        let left = node.left.accept(self)?;
        let right = node.right.accept(self)?;

        match node.operator.kind {
            TokenType::Minus => left.sub(right, operator),
            TokenType::Slash => left.div(right, operator),
            TokenType::Star => left.mul(right, operator),
            TokenType::Plus => left.add(right, operator),
            TokenType::Greater => left.gt(right, operator),
            TokenType::Less => left.lt(right, operator),
            TokenType::GreaterEqual => left.ge(right, operator),
            TokenType::LessEqual => left.le(right, operator),
            TokenType::EqualEqual => left.eq(right, operator),
            TokenType::BangEqual => left.ne(right, operator),
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

        assert_eq!(result, Err(RuntimeError::NumberOperand { operator: token }));
    }
}
