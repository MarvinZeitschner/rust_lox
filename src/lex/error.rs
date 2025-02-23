use thiserror::Error;

#[derive(Error, Debug, PartialEq, PartialOrd, Clone)]
pub enum TokenError {
    #[error("String `{0}` is not terminated")]
    NonTerminatedString(String),

    #[error("Unexpected token `{0}`")]
    UnexpectedToken(String),

    #[error("Unexpected end of file")]
    UnexpectedEOF,
}
