use crate::{gstd::EventEmitter, meta::InterfaceId};

pub trait Service: Sized {
    type Exposure: Exposure;

    fn expose(self, route_idx: u8) -> Self::Exposure;
}

pub trait Exposure {
    fn interface_id() -> InterfaceId;
    fn route_idx(&self) -> u8;
    fn check_asyncness(interface_id: InterfaceId, entry_id: u16) -> Option<bool>;
}

pub trait ExposureWithEvents: Exposure {
    type Events;

    fn emitter(&self) -> EventEmitter<Self::Events> {
        EventEmitter::new(Self::interface_id(), self.route_idx())
    }
}
