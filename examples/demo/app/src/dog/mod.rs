use crate::mammal::MammalService;
use demo_walker::WalkerService;
use sails_rs::prelude::*;

#[event]
#[derive(Clone, Debug, PartialEq, Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum DogEvents {
    Barked,
}

pub struct DogService {
    walker: WalkerService,
    mammal: MammalService,
}

impl DogService {
    pub fn new(walker: WalkerService) -> Self {
        Self {
            walker,
            mammal: MammalService::new(42),
        }
    }
}

// Implementing `Into` for each of the extended services
#[allow(clippy::from_over_into)]
impl Into<(MammalService, WalkerService)> for DogService {
    fn into(self) -> (MammalService, WalkerService) {
        (self.mammal, self.walker)
    }
}

// The resulting Dog service will have 4 methods:
// - MakeSound (from DogService)
// - Walk (from WalkerService)
// - AvgWeight (from MammalService)
// - Position (from WalkerService)
// and 2 events:
// - Barked (from DogEvents)
// - Walked (from WalkerEvents)
// See [IDL](/examples/demo/wasm/demo.idl)
#[service(extends = [MammalService, WalkerService], events = DogEvents)]
impl DogService {
    #[export]
    pub fn make_sound(&mut self) -> &'static str {
        self.emit_event(DogEvents::Barked).unwrap();
        "Woof! Woof!"
    }
}
