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

    #[error("Undefined Variable '{}'", name.line)]
    UndefinedVariable { name: Token<'a> },
}
