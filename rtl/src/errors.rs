use gstd::{errors::Error as GStdError, String};
use thiserror_no_std::Error as ThisError;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(ThisError, Debug, Clone)]
pub enum Error {
    #[error("rtl: {0}")]
    Rtl(#[from] RtlError),
    #[error("gstd: {0}")]
    GStd(#[from] GStdError),
}

#[derive(ThisError, Debug, Clone)]
pub enum RtlError {
    #[error("type `{type_name}` used as event type must be a enum")]
    EventTypeMustBeEnum { type_name: String },
}
