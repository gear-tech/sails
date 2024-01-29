#![no_std]

use gstd::{cell::OnceCell, msg, ActorId};
use sails_exec_context_abstractions::ExecContext;

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
    type ActorId = ActorId;

    fn actor_id(&self) -> &ActorId {
        self.msg_source.get_or_init(msg::source)
    }
}
