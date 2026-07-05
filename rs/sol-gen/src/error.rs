use thiserror::Error;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IDL parse error: {0}")]
    Idl(#[from] sails_idl_parser_v2::error::Error),
    #[error("Conversion type to Solidity error: {0}")]
    Conversion(#[from] crate::sol_conversion::ConversionError),
    #[error("Askama template rendering error: {0}")]
    Askama(#[from] askama::Error),
}
