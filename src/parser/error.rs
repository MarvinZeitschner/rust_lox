use thiserror::Error;

use crate::lex::Token;

#[derive(Error, Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum ParserError<'a> {
    #[error("[line {}] Error: Expected ')' after expression", token.line)]
    UnmatchedParanthesis { token: Token<'a> },

    #[error("[line {}] Error: Expected expression", token.line)]
    ExpectedExpression { token: Token<'a> },

    #[error("[line {}] Error: Unexpected token: {}", token.line, token.lexeme)]
    UnexpectedToken { token: Token<'a> },

    #[error("[line {}] Error: Unexpected end of file", token.line)]
    UnexpectedEOF { token: Token<'a> },

    #[error("{0}")]
    TokenStream(#[from] TokenStreamError),
}

#[derive(Error, Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum TokenStreamError {
    #[error("Internal Parser Error")]
    OutOfBounds,
}
