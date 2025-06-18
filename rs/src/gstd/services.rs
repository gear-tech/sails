use crate::{MessageId, Syscall, Vec, collections::BTreeMap};
use core::ops::DerefMut;

pub trait Service {
    type Exposure: Exposure;
    type BaseExposures;

    fn expose(self, message_id: MessageId, route: &'static [u8]) -> Self::Exposure;
}

pub trait Exposure {
    fn message_id(&self) -> MessageId;
    fn route(&self) -> &'static [u8];

    /// Returns a scope for exposing the service call, which temporarily sets the route into a static    
    fn scope(&self) -> ExposureCallScope {
        ExposureCallScope::new(self.message_id(), self.route())
    }

    /// Returns the route of the service call, which is set after calling [`Exposure::scope`].
    fn scoped_route() -> Option<&'static [u8]> {
        route(Syscall::message_id())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_message_id_to_service_route_map()
-> impl DerefMut<Target = BTreeMap<MessageId, Vec<&'static [u8]>>> {
    use spin::Mutex;

    static MESSAGE_ID_TO_SERVICE_ROUTE: Mutex<BTreeMap<MessageId, Vec<&'static [u8]>>> =
        Mutex::new(BTreeMap::new());

    MESSAGE_ID_TO_SERVICE_ROUTE.lock()
}

#[cfg(target_arch = "wasm32")]
fn get_message_id_to_service_route_map()
-> impl DerefMut<Target = BTreeMap<MessageId, Vec<&'static [u8]>>> {
    static mut MESSAGE_ID_TO_SERVICE_ROUTE: BTreeMap<MessageId, Vec<&'static [u8]>> =
        BTreeMap::new();

    // SAFETY: only for wasm32 target
    #[allow(static_mut_refs)]
    unsafe {
        &mut MESSAGE_ID_TO_SERVICE_ROUTE
    }
}

pub(crate) fn route(message_id: MessageId) -> Option<&'static [u8]> {
    let map = get_message_id_to_service_route_map();
    map.get(&message_id)
        .and_then(|routes| routes.last().copied())
}

/// A scope for exposing a service call, which sets the route into the static `BTreeMap` by message Id.
///
/// When the scope is dropped, it pops the previous route.
#[derive(Clone)]
pub struct ExposureCallScope {
    message_id: MessageId,
    route: &'static [u8],
}

impl ExposureCallScope {
    pub fn new(message_id: MessageId, route: &'static [u8]) -> Self {
        let mut map = get_message_id_to_service_route_map();
        let routes = map.entry(message_id).or_default();
        routes.push(route);
        Self { message_id, route }
    }
}

impl Drop for ExposureCallScope {
    fn drop(&mut self) {
        let mut map = get_message_id_to_service_route_map();
        let routes = map
            .get_mut(&self.message_id)
            .unwrap_or_else(|| unreachable!("Entry for message should always exist"));
        let route = routes
            .pop()
            .unwrap_or_else(|| unreachable!("Route should always exist"));
        if route != self.route {
            unreachable!("Route should always match");
        }
        if routes.is_empty() {
            map.remove(&self.message_id);
        }
    }
}
