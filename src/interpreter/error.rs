use thiserror::Error;

use crate::lex::Token;

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
}
