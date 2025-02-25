use std::ops::{Add, Div, Mul, Neg, Not, Sub};

use crate::{ast::*, lex::TokenType};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
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

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(n) => Value::Number(-n),
            _ => Value::Nil,
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Value::Boolean(b) => Value::Boolean(!b),
            Value::Nil => Value::Boolean(false),
            _ => Value::Boolean(true),
        }
    }
}

impl Sub for Value {
    type Output = Self;

    // TODO: Should strings be parsed?
    // if so: implement error handling for parsing fail
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l - r),
            _ => Value::Nil,
        }
    }
}

impl Div for Value {
    type Output = Self;

    // TODO: Should strings be parsed?
    // if so: implement error handling for parsing fail
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l / r),
            _ => Value::Nil,
        }
    }
}

impl Mul for Value {
    type Output = Self;

    // TODO: Should strings be parsed?
    // if so: implement error handling for parsing fail
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l * r),
            _ => Value::Nil,
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l + r),
            (Value::String(l), Value::String(r)) => Value::String(l + &r),
            (Value::String(l), Value::Number(r)) => Value::String(l + &r.to_string()),
            _ => Value::Nil,
        }
    }
}

pub struct Interpreter;

impl<'a> Visitor<'a, Value> for Interpreter {
    fn visit_literal(&mut self, node: &ExprLiteral) -> Value {
        node.value.clone().into()
    }

    fn visit_grouping(&mut self, node: &ExprGrouping<'a>) -> Value {
        node.value.accept(self)
    }

    fn visit_unary(&mut self, node: &ExprUnary<'a>) -> Value {
        let right = node.value.accept(self);

        match node.operator.kind {
            TokenType::Minus => -(right),
            TokenType::Bang => !(right),
            _ => Value::Nil,
        }
    }

    fn visit_binary(&mut self, node: &ExprBinary<'a>) -> Value {
        let left = node.left.accept(self);
        let right = node.right.accept(self);

        match node.operator.kind {
            TokenType::Minus => left - right,
            TokenType::Slash => left / right,
            TokenType::Star => left * right,
            TokenType::Plus => left + right,
            // TODO: Check if derive handles values correctly, otherwise implement PartialEq and
            // PartialOrd
            TokenType::Greater => Value::Boolean(left > right),
            TokenType::Less => Value::Boolean(left < right),
            TokenType::GreaterEqual => Value::Boolean(left >= right),
            TokenType::LessEqual => Value::Boolean(left <= right),
            // TODO: Add rest of operators
            _ => Value::Nil,
        }
    }
}
