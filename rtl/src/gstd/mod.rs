use crate::{collections::BTreeMap, ActorId, ExecContext, MessageId};
use core::cell::OnceCell;
pub use gstd::{async_init, async_main, handle_signal, message_loop, msg, record_reply};

pub mod calls;
pub mod events;
mod types;

static mut MESSAGE_ID_TO_SERVICE_ROUTE: BTreeMap<MessageId, &'static [u8]> = BTreeMap::new();

pub fn __create_message_scope(service_route: &'static [u8]) -> __MessageScope {
    let msg_id = current_message_id();
    let prev_value = unsafe { MESSAGE_ID_TO_SERVICE_ROUTE.insert(msg_id, service_route) };
    if prev_value.is_some() {
        panic!(
            "Service route already registered for message id: {:?}",
            msg_id
        );
    }
    __MessageScope { msg_id }
}

pub struct __MessageScope {
    msg_id: MessageId,
}

impl Drop for __MessageScope {
    fn drop(&mut self) {
        let removed_value = unsafe { MESSAGE_ID_TO_SERVICE_ROUTE.remove(&self.msg_id) };
        if removed_value.is_none() {
            panic!("Service route not found for message id: {:?}", self.msg_id);
        }
    }
}

fn message_service_route(msg_id: MessageId) -> &'static [u8] {
    let service_route = unsafe { MESSAGE_ID_TO_SERVICE_ROUTE.get(&msg_id).copied() };
    service_route.unwrap_or_else(|| panic!("Service route not found for message id: {:?}", msg_id))
}

fn current_message_id() -> MessageId {
    msg::id().into()
}

#[derive(Default)]
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
