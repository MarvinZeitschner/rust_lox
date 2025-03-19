use strum::EnumDiscriminants;
use thiserror::Error;

use crate::lex::Token;

// TODO: Shard this into smaller domain rule-specific errors
#[derive(Error, Debug, PartialEq, PartialOrd, Clone, EnumDiscriminants)]
#[strum_discriminants(name(ParserErrorContext))]
pub enum ParserError<'a> {
    #[error("[line {}] Error: Expected ')' after expression", token.line)]
    UnmatchedParanthesis { token: Token<'a> },

    #[error("[line {}] Error: Expected '(' after if", token.line)]
    ExpectedLeftParenAfterIf { token: Token<'a> },

    #[error("[line {}] Error: Expected '(' after for", token.line)]
    ExpectedLeftParenAfterFor { token: Token<'a> },

    #[error("[line {}] Error: Expected '(' after while", token.line)]
    ExpectedLeftParenAfterWhile { token: Token<'a> },

    #[error("[line {}] Error: Expected ')' after condition", token.line)]
    ExpectedRightParenAfterCondition { token: Token<'a> },

    #[error("[line {}] Error: Expected ')' after for clauses", token.line)]
    ExpectedRightParenAfterForClause { token: Token<'a> },

    #[error("[line {}] Error: Expected ')' after for arguments", token.line)]
    ExpectedRightParenAfterArguments { token: Token<'a> },

    #[error("[line {}] Error: Expected expression", token.line)]
    ExpectedExpression { token: Token<'a> },

    #[error("[line {}] Error: Expected semicolon", token.line)]
    ExpectedSemicolon { token: Token<'a> },

    #[error("[line {}] Error: Expected function name", token.line)]
    ExpectedFunctionName { token: Token<'a> },

    #[error("[line {}] Error: Expected ( after function name", token.line)]
    ExpectedLeftParenAfterFunctionName { token: Token<'a> },

    #[error("[line {}] Error: Can't have more than 255 parameters", token.line)]
    TooManyFunctionParameters { token: Token<'a> },

    #[error("[line {}] Error: Expected Parameter Name", token.line)]
    ExpectedParameterName { token: Token<'a> },

    #[error("[line {}] Error: Expected ) after parameters", token.line)]
    ExpectedRightParenAfterParameters { token: Token<'a> },

    #[error("[line {}] Error: Expected {{ before function body", token.line)]
    ExpectedLeftBraceBeforeFunctionBody { token: Token<'a> },

    #[error("[line {}] Error: Expected semicolon after loop condition", token.line)]
    ExpectedSemicolonAfterLoopCondition { token: Token<'a> },

    #[error("[line {}] Error: Expected semicolon after return value", token.line)]
    ExpectedSemicolonAfterReturnValue { token: Token<'a> },

    #[error("[line {}] Error: Unexpected token: {}", token.line, token.lexeme)]
    UnexpectedToken { token: Token<'a> },

    #[error("[line {}] Error: Unexpected end of file", token.line)]
    UnexpectedEOF { token: Token<'a> },

    #[error("[line {}] Error: Invalid assignment target", token.line)]
    InvalidAssignmentTarget { token: Token<'a> },

    #[error("[line {}] Error: Can't have more than 255 arguments", token.line)]
    TooManyFunctionArguments { token: Token<'a> },

    #[error("[line {}] Error: Expected {{ after block", token.line)]
    ExpectedRightBrace { token: Token<'a> },

    #[error("[line {}] Expected class name", token.line)]
    ExpectedClassName { token: Token<'a> },

    #[error("[line {}] Expected {{ before class body", token.line)]
    ExpectedLeftBraceBeforeClassBody { token: Token<'a> },

    #[error("[line {}] Expected method", token.line)]
    ExpectedMethod { token: Token<'a> },

    #[error("[line {}] Expected }} after class body", token.line)]
    ExpectedRightBraceAfterClassBody { token: Token<'a> },

    #[error("[line {}] Expected property name after '.'", token.line)]
    ExpectedPropertyNameAfterDot { token: Token<'a> },

    #[error("{0}")]
    TokenStream(#[from] TokenStreamError),
}

impl<'a> ParserErrorContext {
    pub fn to_error(self, token: Token<'a>) -> ParserError<'a> {
        match self {
            ParserErrorContext::UnmatchedParanthesis => ParserError::UnmatchedParanthesis { token },
            ParserErrorContext::ExpectedLeftParenAfterIf => {
                ParserError::ExpectedLeftParenAfterIf { token }
            }
            ParserErrorContext::ExpectedLeftParenAfterFor => {
                ParserError::ExpectedLeftParenAfterFor { token }
            }
            ParserErrorContext::ExpectedRightParenAfterCondition => {
                ParserError::ExpectedRightParenAfterCondition { token }
            }
            ParserErrorContext::ExpectedLeftParenAfterWhile => {
                ParserError::ExpectedLeftParenAfterWhile { token }
            }
            ParserErrorContext::ExpectedExpression => ParserError::ExpectedExpression { token },
            ParserErrorContext::ExpectedSemicolon => ParserError::ExpectedSemicolon { token },
            ParserErrorContext::UnexpectedToken => ParserError::UnexpectedToken { token },
            ParserErrorContext::UnexpectedEOF => ParserError::UnexpectedEOF { token },
            ParserErrorContext::InvalidAssignmentTarget => {
                ParserError::InvalidAssignmentTarget { token }
            }
            ParserErrorContext::ExpectedRightBrace => ParserError::ExpectedRightBrace { token },
            ParserErrorContext::TokenStream => {
                ParserError::TokenStream(TokenStreamError::OutOfBounds)
            }
            ParserErrorContext::ExpectedSemicolonAfterLoopCondition => {
                ParserError::ExpectedSemicolonAfterLoopCondition { token }
            }
            ParserErrorContext::ExpectedRightParenAfterForClause => {
                ParserError::ExpectedRightParenAfterForClause { token }
            }
            ParserErrorContext::ExpectedRightParenAfterArguments => {
                ParserError::ExpectedRightParenAfterArguments { token }
            }
            ParserErrorContext::TooManyFunctionArguments => {
                ParserError::TooManyFunctionArguments { token }
            }
            ParserErrorContext::ExpectedFunctionName => ParserError::ExpectedFunctionName { token },
            ParserErrorContext::ExpectedLeftParenAfterFunctionName => {
                ParserError::ExpectedLeftParenAfterFunctionName { token }
            }
            ParserErrorContext::TooManyFunctionParameters => {
                ParserError::TooManyFunctionParameters { token }
            }
            ParserErrorContext::ExpectedParameterName => {
                ParserError::ExpectedParameterName { token }
            }
            ParserErrorContext::ExpectedRightParenAfterParameters => {
                ParserError::ExpectedRightParenAfterParameters { token }
            }
            ParserErrorContext::ExpectedLeftBraceBeforeFunctionBody => {
                ParserError::ExpectedLeftBraceBeforeFunctionBody { token }
            }
            ParserErrorContext::ExpectedSemicolonAfterReturnValue => {
                ParserError::ExpectedSemicolonAfterReturnValue { token }
            }
            ParserErrorContext::ExpectedClassName => ParserError::ExpectedClassName { token },
            ParserErrorContext::ExpectedMethod => ParserError::ExpectedMethod { token },
            ParserErrorContext::ExpectedRightBraceAfterClassBody => {
                ParserError::ExpectedRightBraceAfterClassBody { token }
            }
            ParserErrorContext::ExpectedLeftBraceBeforeClassBody => {
                ParserError::ExpectedLeftBraceBeforeClassBody { token }
            }
            ParserErrorContext::ExpectedPropertyNameAfterDot => {
                ParserError::ExpectedPropertyNameAfterDot { token }
            }
        }
    }
}

#[derive(Error, Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum TokenStreamError {
    #[error("Internal Parser Error")]
    OutOfBounds,
}
