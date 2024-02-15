#![no_std]

use crate::{cell::OnceCell, gstd::msg};
pub use sails_rtl::*;

pub mod gstd {
    pub use ::gstd::*;
}

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
