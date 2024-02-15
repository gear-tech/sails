#![no_std]

pub use prelude::*;
pub use types::*;

pub mod prelude;
pub mod types;

pub trait ExecContext {
    fn actor_id(&self) -> &ActorId;
}
