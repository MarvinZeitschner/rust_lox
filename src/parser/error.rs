use strum::EnumDiscriminants;
use thiserror::Error;

use crate::lex::Token;

#[derive(Error, Debug, PartialEq, PartialOrd, Clone, Copy, EnumDiscriminants)]
#[strum_discriminants(name(ParserErrorContext))]
pub enum ParserError<'a> {
    #[error("[line {}] Error: Expected ')' after expression", token.line)]
    UnmatchedParanthesis { token: Token<'a> },

    #[error("[line {}] Error: Expected '(' after if", token.line)]
    ExpectedLeftParenAfterIf { token: Token<'a> },

    #[error("[line {}] Error: Expected ')' after condition", token.line)]
    ExpectedRightParenAfterCondition { token: Token<'a> },

    #[error("[line {}] Error: Expected '(' after while", token.line)]
    ExpectedLeftParenAfterWhile { token: Token<'a> },

    #[error("[line {}] Error: Expected expression", token.line)]
    ExpectedExpression { token: Token<'a> },

    #[error("[line {}] Error: Expected semicolon", token.line)]
    ExpectedSemicolon { token: Token<'a> },

    #[error("[line {}] Error: Unexpected token: {}", token.line, token.lexeme)]
    UnexpectedToken { token: Token<'a> },

    #[error("[line {}] Error: Unexpected end of file", token.line)]
    UnexpectedEOF { token: Token<'a> },

    #[error("[line {}] Error: Invalid assignment target", token.line)]
    InvalidAssignmentTarget { token: Token<'a> },

    #[error("[line {}] Error: Expected {{ after block", token.line)]
    ExpectedRightBrace { token: Token<'a> },

    #[error("{0}")]
    TokenStream(#[from] TokenStreamError),
}

#[derive(Error, Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum TokenStreamError {
    #[error("Internal Parser Error")]
    OutOfBounds,
}
