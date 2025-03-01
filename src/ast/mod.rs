pub mod printer;

use crate::lex::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    String(String),
    F64(f64),
    Bool(bool),
    Nil,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    Literal(ExprLiteral),
    Grouping(ExprGrouping<'a>),
    Unary(ExprUnary<'a>),
    Binary(ExprBinary<'a>),
}

impl<'a> Expr<'a> {
    pub fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        match self {
            Expr::Literal(node) => visitor.visit_literal(node),
            Expr::Grouping(node) => visitor.visit_grouping(node),
            Expr::Unary(node) => visitor.visit_unary(node),
            Expr::Binary(node) => visitor.visit_binary(node),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExprLiteral {
    pub value: LiteralValue,
}

impl ExprLiteral {
    pub fn new(value: LiteralValue) -> Self {
        Self { value }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExprGrouping<'a> {
    pub value: Box<Expr<'a>>,
}

impl<'a> ExprGrouping<'a> {
    pub fn new(value: Box<Expr<'a>>) -> Self {
        Self { value }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExprUnary<'a> {
    pub operator: Token<'a>,
    pub value: Box<Expr<'a>>,
}

impl<'a> ExprUnary<'a> {
    pub fn new(operator: Token<'a>, value: Box<Expr<'a>>) -> Self {
        Self { operator, value }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExprBinary<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: Token<'a>,
    pub right: Box<Expr<'a>>,
}

impl<'a> ExprBinary<'a> {
    pub fn new(left: Box<Expr<'a>>, operator: Token<'a>, right: Box<Expr<'a>>) -> Self {
        Self {
            left,
            operator,
            right,
        }
    }
}

pub trait Visitor {
    type Result;

    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::Result;
    fn visit_grouping(&mut self, expr: &ExprGrouping) -> Self::Result;
    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::Result;
    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::Result;
}
