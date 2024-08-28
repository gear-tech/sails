#[cfg(not(feature = "ethexe"))]
#[doc(hidden)]
pub use gstd::handle_signal;
#[doc(hidden)]
pub use gstd::{async_init, async_main, handle_reply_with_hook, message_loop};
pub use gstd::{debug, msg};
pub use sails_macros::*;

use crate::{ActorId, MessageId};
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
