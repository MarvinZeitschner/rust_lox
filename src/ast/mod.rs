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
#[name = "Expr"]
pub enum Expression<'a> {
    Literal {
        value: LiteralValue,
    },
    Grouping {
        value: Box<Expr<'a>>,
    },
    Logical {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
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
    Call {
        callee: Box<Expr<'a>>,
        paren: Token<'a>,
        arguments: Vec<Expr<'a>>,
    },
    Assign {
        name: Token<'a>,
        value: Box<Expr<'a>>,
    },
    Variable {
        name: Token<'a>,
    },
}

#[derive(Ast, Debug, PartialEq)]
#[name = "Stmt"]
pub enum Statement<'a> {
    Block {
        statements: Vec<Stmt<'a>>,
    },
    Expression {
        expr: Expr<'a>,
    },
    If {
        condition: Expr<'a>,
        then_branch: Box<Stmt<'a>>,
        else_branch: Option<Box<Stmt<'a>>>,
    },
    Print {
        expr: Expr<'a>,
    },
    Var {
        name: Token<'a>,
        initializer: Option<Expr<'a>>,
    },
    While {
        condition: Expr<'a>,
        body: Box<Stmt<'a>>,
    },
}
