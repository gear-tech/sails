use super::*;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("meta is invalid: {0}")]
    MetaIsInvalid(String),
    #[error("type id `{0}` is not found in the type registry")]
    TypeIdIsUnknown(u32),
    #[error("type `{0}` is not supported")]
    TypeIsUnsupported(String),
    #[error(transparent)]
    Template(#[from] askama::Error),
    #[cfg(feature = "std")]
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
