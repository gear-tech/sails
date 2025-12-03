use alloc::string::{String, ToString};
use thiserror::Error;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Error, PartialEq)]
pub enum RuleError {
    #[error("Expected {0}")]
    Expected(String),
    #[error("Unexpected {0}")]
    Unexpected(String),
}

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("{0:?}")]
    Pest(PestErrorFormatter),
    #[error("Rule error: {0}")]
    Rule(#[from] RuleError),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

// A newtype wrapper for `pest::error::Error` to provide a custom `Debug`
// implementation. This ensures that the formatted error string from `pest`
// (which includes newlines and indentation) is displayed directly when `Debug`
// formatting is requested (e.g., in panic messages or `dbg!`), rather than
// being escaped. This allows for clean, readable error output.
#[derive(PartialEq)]
pub struct PestErrorFormatter(String);

impl core::fmt::Debug for PestErrorFormatter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<pest::error::Error<crate::Rule>> for Error {
    fn from(e: pest::error::Error<crate::Rule>) -> Self {
        Error::Pest(PestErrorFormatter(e.to_string()))
    }
}
