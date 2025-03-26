pub mod printer;

use std::hash::Hash;
use std::hash::Hasher;

use ast_macro::Ast;

use crate::lex::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    String(String),
    F64(f64),
    Bool(bool),
    Nil,
}

impl Hash for LiteralValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        if let LiteralValue::F64(n) = self {
            n.to_bits().hash(state);
        }
    }
}

impl Eq for LiteralValue {}

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
    },
    This {
        keyword: Token<'a>,
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
