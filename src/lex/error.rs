use thiserror::Error;

#[derive(Error, Debug, PartialEq, PartialOrd, Clone)]
pub enum TokenError {
    #[error("[line {line}] Error: String `{string}` is not terminated")]
    NonTerminatedString { line: u32, string: String },

    #[error("[line {line}] Error: Unexpected token `{token}`")]
    UnexpectedToken { line: u32, token: String },

    #[error("Unexpected end of file")]
    UnexpectedEOF,
}
