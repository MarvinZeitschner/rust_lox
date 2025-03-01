use thiserror::Error;

#[derive(Error, Debug, PartialEq, PartialOrd, Clone)]
pub enum RuntimeError {
    #[error("[line ] Operand must be a number")]
    InvalidOperator,
}
