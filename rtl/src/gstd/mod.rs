use crate::{ActorId, MessageId};
use core::cell::OnceCell;
#[doc(hidden)]
pub use gstd::{async_init, async_main, handle_signal, message_loop, msg, record_reply};
pub use sails_macros::*;

pub mod calls;
pub mod events;
pub mod services;
mod types;

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
        *self.msg_source.get_or_init(|| msg::source().into())
    }

    fn message_id(&self) -> MessageId {
        *self.msg_id.get_or_init(|| msg::id().into())
    }
}
