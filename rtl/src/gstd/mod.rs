use crate::{ActorId, ExecContext};
use core::cell::OnceCell;
pub use gstd::{async_init, async_main, handle_signal, message_loop, msg, record_reply};

pub mod calls;
pub mod events;
mod types;

#[derive(Default)]
pub struct GStdExecContext {
    msg_source: OnceCell<ActorId>,
}

impl GStdExecContext {
    pub fn new() -> Self {
        Self {
            msg_source: OnceCell::new(),
        }
    }
}

impl ExecContext for GStdExecContext {
    fn actor_id(&self) -> &ActorId {
        self.msg_source
            .get_or_init(|| msg::source().as_ref().into())
    }
}
