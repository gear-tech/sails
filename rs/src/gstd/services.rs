use crate::{gstd::EventEmitter, meta::InterfaceId};

pub trait Service {
    type Exposure: Exposure;

    fn expose(self, route: &'static [u8]) -> Self::Exposure;
}

pub trait Exposure {
    fn route(&self) -> &'static [u8];
    fn check_asyncness(interface_id: InterfaceId, entry_id: u16) -> Option<bool>;
}

pub trait ExposureWithEvents: Exposure {
    type Events;

    fn emitter(&self) -> EventEmitter<Self::Events> {
        let route = self.route();
        EventEmitter::new(route)
    }
}
