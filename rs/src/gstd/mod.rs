use crate::{ActorId, MessageId};
use core::cell::OnceCell;
#[doc(hidden)]
pub use gstd::{async_init, async_main, handle_reply_with_hook, handle_signal, message_loop, msg};
pub use sails_macros::*;

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
