pub use core::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128};
pub use gprimitives::{ActorId, CodeId, H160, H256, MessageId, NonZeroU256, U256};
#[cfg(feature = "gstd")]
pub use gstd::BlockCount;

pub type ValueUnit = u128;

pub type GasUnit = u64;
