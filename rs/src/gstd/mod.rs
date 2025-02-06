#[cfg(not(feature = "ethexe"))]
#[doc(hidden)]
pub use gstd::handle_signal;
#[doc(hidden)]
pub use gstd::{async_init, async_main, handle_reply_with_hook, message_loop};
pub use gstd::{debug, exec, msg};
pub use sails_macros::*;

use crate::prelude::*;
use core::cell::OnceCell;

pub mod calls;
pub mod events;
pub mod services;

// TODO: To be renamed into SysCalls or something similar
pub trait ExecContext {
    fn actor_id(&self) -> ActorId;

    fn message_id(&self) -> MessageId;
}

#[derive(Default, Clone)]
pub struct GStdExecContext {
    msg_source: OnceCell<ActorId>,
    msg_id: OnceCell<MessageId>,
}

impl GStdExecContext {
    pub fn new() -> Self {
        Self {
            msg_source: OnceCell::new(),
            msg_id: OnceCell::new(),
        }
    }
}

impl ExecContext for GStdExecContext {
    fn actor_id(&self) -> ActorId {
        *self.msg_source.get_or_init(msg::source)
    }

    fn message_id(&self) -> MessageId {
        *self.msg_id.get_or_init(msg::id)
    }
}

pub struct CommandReply<T>(T, ValueUnit);

impl<T> CommandReply<T> {
    pub fn new(result: T) -> Self {
        Self(result, 0)
    }

    pub fn with_value(self, value: ValueUnit) -> Self {
        Self(self.0, value)
    }

    pub fn to_tuple(self) -> (T, ValueUnit) {
        (self.0, self.1)
    }
}

impl<T> From<T> for CommandReply<T> {
    fn from(result: T) -> Self {
        Self(result, 0)
    }
}

impl<T> From<(T, ValueUnit)> for CommandReply<T> {
    fn from(value: (T, ValueUnit)) -> Self {
        Self(value.0, value.1)
    }
}

pub fn unknown_input_panic(message: &str, input: &[u8]) -> ! {
    let mut __input = input;
    let input: String = crate::Decode::decode(&mut __input).unwrap_or_else(|_| {
        if input.len() <= 8 {
            format!("0x{}", crate::hex::encode(input))
        } else {
            format!(
                "0x{}..{}",
                crate::hex::encode(&input[..4]),
                crate::hex::encode(&input[input.len() - 4..])
            )
        }
    });
    panic!("{}: {}", message, input)
}
