pub mod printer;

use ast_macro::Ast;

use crate::lex::lexer::Token;

#[derive(Debug)]
pub enum LiteralValue {
    String(String),
    F64(f64),
    Bool(bool),
    Nil,
}

#[derive(Ast, Debug)]
pub enum Expression {
    Literal {
        value: LiteralValue,
    },
    Grouping {
        value: Box<Expr>,
    },
    Unary {
        operator: String,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: &'a Token,
        right: Box<Expr>,
    },
}
