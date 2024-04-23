#![no_std]

pub use prelude::*;

pub mod calls;
pub mod errors;
pub mod gstd;
#[cfg(not(target_arch = "wasm32"))]
pub mod gtest;
pub mod prelude;
pub mod types;

pub mod hex {
    pub use ::hex::*;
}

pub mod scale {
    pub use parity_scale_codec::{Decode, Encode, EncodeLike};
    pub use scale_info::TypeInfo;
}

pub trait ExecContext {
    fn actor_id(&self) -> ActorId;

    fn message_id(&self) -> MessageId;
}
