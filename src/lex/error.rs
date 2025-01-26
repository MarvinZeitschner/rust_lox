use core::fmt;
use std::error::Error;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum TokenError {}

impl Error for TokenError {}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenError")
    }
}
