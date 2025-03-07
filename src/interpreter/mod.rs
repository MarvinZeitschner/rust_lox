pub mod environment;
pub mod error;

use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Not, Sub},
};

use environment::{Environment, EnvironmentBuilder};
use error::RuntimeError;

use crate::{
    ast::*,
    lex::{Token, TokenType},
};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Number(_) => true,
            Value::String(_) => true,
            Value::Boolean(b) => *b,
            Value::Nil => false,
        }
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self {
        match self {
            Value::Number(n) => Value::Number(-n),
            _ => unreachable!(),
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Value::Boolean(b) => Value::Boolean(!b),
            Value::Number(_) => Value::Boolean(false),
            Value::String(_) => Value::Boolean(false),
            Value::Nil => Value::Boolean(true),
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l + r),
            (Value::String(l), Value::String(r)) => Value::String(l + &r),
            _ => unreachable!(),
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l - r),
            _ => unreachable!(),
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l / r),
            _ => unreachable!(),
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l * r),
            _ => unreachable!(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => l.partial_cmp(r),
            (Value::String(l), Value::String(r)) => l.partial_cmp(r),
            (Value::Boolean(l), Value::Boolean(r)) => l.partial_cmp(r),
            (Value::Nil, Value::Nil) => Some(std::cmp::Ordering::Equal),
            _ => None,
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

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Default)]
pub struct Interpreter<'a> {
    environment: Environment<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt<'a>>) -> Result<(), RuntimeError<'a>> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt<'a>) -> Result<(), RuntimeError<'a>> {
        stmt.accept(self)
    }

    fn execute_block(
        &mut self,
        statements: Vec<Stmt<'a>>,
        environment: Environment<'a>,
    ) -> Result<(), RuntimeError<'a>> {
        let previous_env = self.environment.clone();

        self.environment = environment;

        for stmt in statements {
            match self.execute(stmt) {
                Ok(_) => {}
                Err(e) => {
                    self.environment = previous_env;
                    return Err(e);
                }
            }
        }

        self.environment = previous_env;

        Ok(())
    }

    fn evaluate(&mut self, expr: Expr<'a>) -> Result<Value, RuntimeError<'a>> {
        expr.accept(self)
    }

    fn check_number_operand(
        &mut self,
        value: &Value,
        operator: Token<'a>,
    ) -> Result<(), RuntimeError<'a>> {
        match value {
            Value::Number(_) => Ok(()),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn check_number_operands(
        &mut self,
        left: &Value,
        right: &Value,
        operator: Token<'a>,
    ) -> Result<(), RuntimeError<'a>> {
        match (left, right) {
            (Value::Number(_), Value::Number(_)) => Ok(()),
            _ => Err(RuntimeError::MutlipleNumberOperands { operator }),
        }
    }
}

impl<'a> ExprVisitor<'a> for Interpreter<'a> {
    type Output = Result<Value, RuntimeError<'a>>;

    fn visit_literal(&mut self, node: ExprLiteral) -> Self::Output {
        Ok(node.value.into())
    }

    fn visit_grouping(&mut self, node: ExprGrouping<'a>) -> Self::Output {
        self.evaluate(*node.value)
    }

    fn visit_logical(&mut self, node: ExprLogical<'a>) -> Self::Output {
        let left = self.evaluate(*node.left)?;

        if node.operator.kind == TokenType::Or && left.is_truthy() {
            return Ok(left);
        } else if node.operator.kind == TokenType::And && !left.is_truthy() {
            return Ok(left);
        }

        self.evaluate(*node.right)
    }

    fn visit_unary(&mut self, node: ExprUnary<'a>) -> Self::Output {
        let operator = node.operator;
        let right = self.evaluate(*node.value)?;

        match node.operator.kind {
            TokenType::Minus => {
                self.check_number_operand(&right, operator)?;
                Ok(-right)
            }
            TokenType::Bang => Ok(!right),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn visit_binary(&mut self, node: ExprBinary<'a>) -> Self::Output {
        let operator = node.operator;
        let left = self.evaluate(*node.left)?;
        let right = self.evaluate(*node.right)?;

        match node.operator.kind {
            TokenType::Minus => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(left - right)
            }
            TokenType::Slash => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(left / right)
            }
            TokenType::Star => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(left * right)
            }
            TokenType::Plus => {
                if let (Value::String(_), Value::String(_)) = (&left, &right) {
                    return Ok(left + right);
                }
                if let (Value::Number(_), Value::Number(_)) = (&left, &right) {
                    return Ok(left + right);
                }
                Err(RuntimeError::NumberOrStringOperands { operator })
            }
            TokenType::Greater => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(Value::Boolean(left > right))
            }
            TokenType::Less => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(Value::Boolean(left < right))
            }
            TokenType::GreaterEqual => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(Value::Boolean(left >= right))
            }
            TokenType::LessEqual => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(Value::Boolean(left <= right))
            }
            TokenType::EqualEqual => Ok(Value::Boolean(left == right)),
            TokenType::BangEqual => Ok(Value::Boolean(left != right)),
            _ => Ok(Value::Nil),
        }
    }

    fn visit_assign(&mut self, node: ExprAssign<'a>) -> Self::Output {
        let value = self.evaluate(*node.value)?;
        self.environment.assign(node.name, value.clone())?;
        Ok(value)
    }

    fn visit_variable(&mut self, node: ExprVariable<'a>) -> Self::Output {
        self.environment.get(node.name)
    }
}

impl<'a> StmtVisitor<'a> for Interpreter<'a> {
    type Output = Result<(), RuntimeError<'a>>;

    fn visit_block(&mut self, node: StmtBlock<'a>) -> Self::Output {
        self.execute_block(
            node.statements,
            EnvironmentBuilder::new()
                .enclosing(self.environment.clone())
                .build(),
        )?;
        Ok(())
    }

    fn visit_expression(&mut self, node: StmtExpression<'a>) -> Self::Output {
        self.evaluate(node.expr)?;
        Ok(())
    }

    fn visit_if(&mut self, node: StmtIf<'a>) -> Self::Output {
        let condition = self.evaluate(node.condition)?;
        let Value::Boolean(condition) = condition else {
            // TODO:
            return Ok(());
        };
        if condition {
            self.execute(*node.then_branch)?;
        } else if node.else_branch.is_some() {
            self.execute(*node.else_branch.unwrap())?;
        }

        Ok(())
    }

    fn visit_print(&mut self, node: StmtPrint<'a>) -> Self::Output {
        let value = self.evaluate(node.expr)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var(&mut self, node: StmtVar<'a>) -> Self::Output {
        let mut value = None;
        if let Some(initializer) = node.initializer {
            value = Some(self.evaluate(initializer)?);
        }
        self.environment.define(node.name.lexeme, value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::lex::{Span, Token};

    use super::*;

    #[test]
    fn literal() {
        let mut interpreter = Interpreter::new();

        let expr = Expr::Literal(ExprLiteral::new(LiteralValue::F64(1.0)));
        let result = interpreter.evaluate(expr).unwrap();

        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn grouping() {
        let mut interpreter = Interpreter::new();

        let expr = Expr::Grouping(ExprGrouping::new(Box::new(Expr::Literal(
            ExprLiteral::new(LiteralValue::F64(1.0)),
        ))));
        let result = interpreter.evaluate(expr).unwrap();

        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn unary() {
        let mut interpreter = Interpreter::new();

        let span = Span { begin: 0, end: 1 };
        let token = Token::new(TokenType::Minus, "-", 1, span);
        let expr = Expr::Unary(ExprUnary::new(
            token,
            Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::F64(1.0)))),
        ));
        let result = interpreter.evaluate(expr).unwrap();

        assert_eq!(result, Value::Number(-1.0));
    }

    #[test]
    fn error() {
        let mut interpreter = Interpreter::new();

        let span = Span { begin: 0, end: 1 };
        let token = Token::new(TokenType::Minus, "-", 1, span);
        let expr = Expr::Unary(ExprUnary::new(
            token,
            Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::String(
                "1".to_string(),
            )))),
        ));
        let result = interpreter.evaluate(expr);

        assert_eq!(result, Err(RuntimeError::NumberOperand { operator: token }));
    }
}
