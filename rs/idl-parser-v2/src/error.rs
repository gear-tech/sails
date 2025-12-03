use alloc::string::{String, ToString};
use thiserror::Error;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("{0:?}")]
    Pest(PestErrorFormatter),
    #[error("Expected rule '{expected:?}', but found '{found:?}'. {message:?}")]
    ExpectedRule {
        expected: String,
        found: String,
        message: Option<String>,
    },
    #[error("Expected next rule or identifier. {0:?}")]
    ExpectedNext(Option<String>),
    #[error("Unexpected rule: {0:?}")]
    UnexpectedRule(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Expected at most one program per IDL document")]
    MultiplePrograms,
    #[error("Invalid primitive type: {0}")]
    InvalidPrimitiveType(String),
    #[error("An internal error occurred: {0}")]
    Internal(String),
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
