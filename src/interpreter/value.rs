use std::fmt::{self};

use crate::ast::{Expr, LiteralValue};

use super::{error::RuntimeError, Interpreter};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

pub trait LoxCallable<'a> {
    fn call(
        &self,
        interpreter: &mut Interpreter<'a>,
        arguments: Vec<&Expr<'a>>,
    ) -> Result<Value<'a>, RuntimeError<'a>>;
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

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Number(f64),
    String(String),
    Boolean(bool),
    Callable(Box<dyn LoxCallable<'a>>),
    Nil,
}

impl<'a> Value<'a> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Number(_) => true,
            Value::String(_) => true,
            Value::Boolean(b) => *b,
            Value::Callable(_) => true,
            Value::Nil => false,
        }
    }
}

impl<'a> Neg for Value<'a> {
    type Output = Self;

    fn neg(self) -> Self {
        match self {
            Value::Number(n) => Value::Number(-n),
            _ => unreachable!(),
        }
    }
}

impl<'a> Not for Value<'a> {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Value::Boolean(b) => Value::Boolean(!b),
            Value::Number(_) => Value::Boolean(false),
            Value::String(_) => Value::Boolean(false),
            Value::Callable(_) => Value::Boolean(false),
            Value::Nil => Value::Boolean(true),
        }
    }
}

impl<'a> Add for Value<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l + r),
            (Value::String(l), Value::String(r)) => Value::String(l + &r),
            _ => unreachable!(),
        }
    }
}

impl<'a> Sub for Value<'a> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l - r),
            _ => unreachable!(),
        }
    }
}

impl<'a> Div for Value<'a> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l / r),
            _ => unreachable!(),
        }
    }
}

impl<'a> Mul for Value<'a> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l * r),
            _ => unreachable!(),
        }
    }
}

impl<'a> PartialEq for Value<'a> {
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

impl<'a> PartialOrd for Value<'a> {
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

impl<'a> From<LiteralValue> for Value<'a> {
    fn from(literal: LiteralValue) -> Self {
        match literal {
            LiteralValue::F64(f) => Value::Number(f),
            LiteralValue::String(s) => Value::String(s),
            LiteralValue::Bool(b) => Value::Boolean(b),
            LiteralValue::Nil => Value::Nil,
        }
    }
}

impl<'a> fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Callable(lox_callable) => write!(f, "{:?}", lox_callable),
            Value::Nil => write!(f, "nil"),
        }
    }
}
