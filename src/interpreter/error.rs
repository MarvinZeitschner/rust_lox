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

    #[error("[Undefined Variable '{}'", name.lexeme)]
    UndefinedVariable { name: Token<'a> },

    #[error("[line {}] Can only call functions and classes", token.line)]
    NotCallable { token: Token<'a> },

    #[error("[line {}] Expected {} arguments but got {}", token.line, expected_arity, given_len)]
    ArgumentCount {
        token: Token<'a>,
        expected_arity: usize,
        given_len: usize,
    },

    #[error("{0}")]
    CallableError(#[from] CallableError),

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

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Return<'a> {
    pub value: Option<Value<'a>>,
}
