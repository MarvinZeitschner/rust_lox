use thiserror::Error;

use crate::lex::Token;

use super::value::Value;

#[derive(Error, Debug, PartialEq, PartialOrd, Clone)]
pub enum RuntimeError<'a> {
    #[error("[line {}] Operand must be a number", operator.line)]
    NumberOperand { operator: Token<'a> },

    #[error("[line {}] Operands must be a number", operator.line)]
    MutlipleNumberOperands { operator: Token<'a> },

    #[error("[line {}] Operands must be two numbers or two strings", operator.line)]
    NumberOrStringOperands { operator: Token<'a> },

    #[error("[line {}] Undefined Variable '{}'", name.line, name.lexeme)]
    UndefinedVariable { name: Token<'a> },

    #[error("[line {}] Can only call functions and classes", token.line)]
    NotCallable { token: Token<'a> },

    #[error("[line {}] Expected {} arguments but got {}", token.line, expected_arity, given_len)]
    ArgumentCount {
        token: Token<'a>,
        expected_arity: usize,
        given_len: usize,
    },

    #[error("Internal Error: Error while creating environments")]
    EnvironmentCreationError,

    #[error("{0}")]
    CallableError(#[from] CallableError),

    #[error("{0}")]
    ClassError(ClassError<'a>),

    #[error("{0}")]
    ResolverError(#[from] ResolverError<'static>),

    // Not an actual Error, but rather a special type to unwind the interpreter to the call method of LoxCallable when a value is returned
    #[error("Internal Error: Unhandled return")]
    Return(Return<'a>),
}

#[derive(Error, Debug, PartialEq, PartialOrd, Clone)]
pub enum CallableError {
    #[error("Internal Error")]
    InternalError,

    #[error("Parameter not Found; Internal Error")]
    ParamNotFound,
}

#[derive(Error, Debug, PartialEq, PartialOrd, Clone)]
pub enum ClassError<'a> {
    #[error("[line {}] Only instances have properties", token.line)]
    InvalidPropertyAccess { token: Token<'a> },

    #[error("[line {}] Undefined property {}", token.line, token.lexeme)]
    UndefinedProperty { token: Token<'a> },
}

#[derive(Error, Debug, PartialEq, PartialOrd, Clone)]
pub enum ResolverError<'a> {
    #[error("[line {}] Can't read local variable in its own initializer", token.line)]
    VariableInOwnInitializer { token: Token<'a> },

    #[error("Internal Error")]
    InternalResolverError,

    #[error("[line {}] Already a variable with the same name in the scope", token.line)]
    SameNameVariableInLocalScope { token: Token<'a> },

    #[error("[line {}] Cannot return from top-level code", token.line)]
    TopLevelReturn { token: Token<'a> },

    #[error("[line {}] Cannot use 'this' outside a class", token.line)]
    ThisOutsideClass { token: Token<'a> },

    #[error("[line {}] Cannot return a value from a constructor", token.line)]
    ReturnInConstructor { token: Token<'a> },
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Return<'a> {
    pub value: Option<Value<'a>>,
}
