#[doc(hidden)]
pub use gstd::{async_init, async_main, handle_reply_with_hook, handle_signal, message_loop};
pub use gstd::{debug, exec, msg};
pub use sails_macros::*;

use crate::{ActorId, MessageId, ValueUnit};
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

pub struct CommandResult<T>(T, ValueUnit);

impl<T> CommandResult<T> {
    pub fn new(value: T) -> Self {
        Self(value, 0)
    }

    pub fn with_value(self, value: ValueUnit) -> Self {
        Self(self.0, value)
    }

    pub fn to_tuple(self) -> (T, ValueUnit) {
        (self.0, self.1)
    }
}

impl<T> From<T> for CommandResult<T> {
    fn from(value: T) -> Self {
        Self(value, 0)
    }
}

impl<T> From<(T, ValueUnit)> for CommandResult<T> {
    fn from(value: (T, ValueUnit)) -> Self {
        Self(value.0, value.1)
    }
}
