#![no_std]

pub use prelude::*;

pub mod errors;
pub mod gstd;
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
    fn actor_id(&self) -> &ActorId;
}
