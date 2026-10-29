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

/// Where the resolver found a local variable: `depth` environments up from the use, at position
/// `slot` in that environment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Local {
    pub depth: usize,
    pub slot: usize,
}

/// `local` is filled in by the resolver. `None` means the variable is global.
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
        local: Cell<Option<Local>>,
    },
    This {
        keyword: Token<'a>,
        local: Cell<Option<Local>>,
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
        local: Cell<Option<Local>>,
    },
    Variable {
        name: Token<'a>,
        local: Cell<Option<Local>>,
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
