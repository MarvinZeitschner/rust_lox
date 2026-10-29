pub mod printer;

use std::cell::Cell;

use ast_macro::Ast;

use crate::lex::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    String(String),
    F64(f64),
    Bool(bool),
    Nil,
}

/// `depth` is filled in by the resolver: the number of environments between the use of a local
/// variable and its declaration. `None` means the variable is global.
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
    Set {
        object: Box<Expr<'a>>,
        name: Token<'a>,
        value: Box<Expr<'a>>,
    },
    Super {
        keyword: Token<'a>,
        method: Token<'a>,
        depth: Cell<Option<usize>>,
    },
    This {
        keyword: Token<'a>,
        depth: Cell<Option<usize>>,
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
    Get {
        object: Box<Expr<'a>>,
        name: Token<'a>,
    },
    Assign {
        name: Token<'a>,
        value: Box<Expr<'a>>,
        depth: Cell<Option<usize>>,
    },
    Variable {
        name: Token<'a>,
        depth: Cell<Option<usize>>,
    },
}

#[derive(Ast, Debug, PartialEq)]
#[name = "Stmt"]
pub enum Statement<'a> {
    Block {
        statements: Vec<Stmt<'a>>,
    },
    Class {
        name: Token<'a>,
        superclass: Option<Expr<'a>>,
        methods: Vec<StmtFunction<'a>>,
    },
    Expression {
        expr: Expr<'a>,
    },
    Function {
        name: Token<'a>,
        params: Vec<Token<'a>>,
        body: Vec<Stmt<'a>>,
    },
    If {
        condition: Expr<'a>,
        then_branch: Box<Stmt<'a>>,
        else_branch: Option<Box<Stmt<'a>>>,
    },
    Print {
        expr: Expr<'a>,
    },
    Return {
        keyword: Token<'a>,
        value: Option<Expr<'a>>,
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
