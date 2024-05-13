use crate::{collections::BTreeMap, MessageId, Vec};

static mut MESSAGE_ID_TO_SERVICE_ROUTE: BTreeMap<MessageId, Vec<&'static [u8]>> = BTreeMap::new();

pub trait Service {
    type Exposure: Exposure;

    fn expose(self, message_id: MessageId, route: &'static [u8]) -> Self::Exposure;
}

pub trait Exposure {
    fn message_id(&self) -> MessageId;
    fn route(&self) -> &'static [u8];
}

#[derive(Debug, Clone, Copy)]
pub struct ExposureContext {
    message_id: MessageId,
    route: &'static [u8],
}

impl ExposureContext {
    pub fn message_id(&self) -> MessageId {
        self.message_id
    }

    pub fn route(&self) -> &'static [u8] {
        self.route
    }
}

pub(crate) fn exposure_context(message_id: MessageId) -> ExposureContext {
    let route = unsafe {
        MESSAGE_ID_TO_SERVICE_ROUTE
            .get(&message_id)
            .and_then(|routes| routes.last().copied())
            .unwrap_or_else(|| {
                panic!(
                    "Exposure context is not found for message id {:?}",
                    message_id
                )
            })
    };
    ExposureContext { message_id, route }
}

pub struct ExposureCallScope {
    message_id: MessageId,
    route: &'static [u8],
}

impl ExposureCallScope {
    pub fn new(exposure: &impl Exposure) -> Self {
        let routes = unsafe {
            MESSAGE_ID_TO_SERVICE_ROUTE
                .entry(exposure.message_id())
                .or_default()
        };
        routes.push(exposure.route());
        Self {
            message_id: exposure.message_id(),
            route: exposure.route(),
        }
    }
}

impl Drop for ExposureCallScope {
    fn drop(&mut self) {
        let routes = unsafe {
            MESSAGE_ID_TO_SERVICE_ROUTE
                .get_mut(&self.message_id)
                .unwrap_or_else(|| unreachable!("Entry for message should always exist"))
        };
        let route = routes
            .pop()
            .unwrap_or_else(|| unreachable!("Route should always exist"));
        if route != self.route {
            unreachable!("Route should always match");
        }
        if routes.is_empty() {
            unsafe { MESSAGE_ID_TO_SERVICE_ROUTE.remove(&self.message_id) };
        }
    }
}
