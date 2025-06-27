use crate::gstd::EventEmitter;

pub trait Service {
    type Exposure: Exposure;

    fn expose(self, route: &'static [u8]) -> Self::Exposure;
}

pub trait Exposure {
    fn route(&self) -> &'static [u8];
    fn check_asyncness(input: &[u8]) -> Option<bool>;
}

pub trait ExposureWithEvents: Exposure {
    type Events;

    fn emitter(&self) -> EventEmitter<Self::Events> {
        let route = self.route();
        EventEmitter::new(route)
    }
}
