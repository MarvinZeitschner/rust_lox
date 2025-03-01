pub mod printer;

use ast_macro::Ast;

use crate::lex::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    String(String),
    F64(f64),
    Bool(bool),
    Nil,
}

#[derive(Ast, Debug, PartialEq)]
pub enum Expression<'a> {
    Grouping {
        value: Box<Expr<'a>>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token<'a>,
        value: Box<Expr<'a>>,
    },
    Binary {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
}
