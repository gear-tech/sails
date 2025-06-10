use crate::MessageId;

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
        ExposureCallScope::new(self.route())
    }
}

#[cfg(not(feature = "std"))]
static ROUTE_CELL: crate::gstd::utils::SyncCell<Option<&'static [u8]>> =
    crate::gstd::utils::SyncCell::new(None);

#[cfg(feature = "std")]
std::thread_local! {
    static ROUTE_CELL: core::cell::Cell<Option<&'static [u8]>> = const { core::cell::Cell::new(None) };
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn route() -> Option<&'static [u8]> {
    ROUTE_CELL.get()
}

/// A scope for exposing a service call, which temporarily sets the route into the static `Cell`,
/// and stores the previous route.
///
/// When the scope is dropped, it restores the previous route.
pub struct ExposureCallScope {
    prev_route: Option<&'static [u8]>,
}

impl ExposureCallScope {
    pub fn new(route: &'static [u8]) -> Self {
        let prev_route = ROUTE_CELL.replace(Some(route));
        Self { prev_route }
    }
}

impl Drop for ExposureCallScope {
    fn drop(&mut self) {
        _ = ROUTE_CELL.replace(self.prev_route);
    }
}
