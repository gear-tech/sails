use gear_core_errors::ErrorReplyReason;
use gstd::{
    errors::{CoreError as GCoreError, Error as GStdError},
    String,
};
use parity_scale_codec::Error as CodecError;
use thiserror_no_std::Error as ThisError;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("rtl: {0}")]
    Rtl(#[from] RtlError),
    #[error("gstd: {0}")]
    GStd(#[from] GStdError),
    #[error("gcore: {0}")]
    GCore(#[from] GCoreError),
    #[error("codec: {0}")]
    Codec(#[from] CodecError),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("codec: {0}")]
    GClient(#[from] gclient::Error),
}

#[derive(ThisError, Debug, Clone)]
pub enum RtlError {
    #[error("type `{type_name}` used as event type must be a enum")]
    EventTypeMustBeEnum { type_name: String },
    #[error("reply prefix mismatches")]
    ReplyPrefixMismatches, // TODO: add some context about the received reply, some encoded hex
    #[error("reply is missing")]
    ReplyIsMissing,
    #[error("reply is ambiguous")]
    ReplyIsAmbiguous,
    #[error("reply code is missing")]
    ReplyCodeIsMissing,
    #[error("reply error: {0}")]
    ReplyHasError(ErrorReplyReason),
    #[error("program code is not found")]
    ProgramCodeIsNotFound,
    #[error("program is not found")]
    ProgramIsNotFound,
    #[error("actor is not set")]
    ActorIsNotSet,
    #[error("reply has error string")]
    ReplyHasErrorString(String),
}
