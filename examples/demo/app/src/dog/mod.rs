use crate::mammal::MammalService;
use demo_walker::WalkerService;
use sails::prelude::*;

#[derive(Encode, TypeInfo)]
#[codec(crate = sails::scale_codec)]
#[scale_info(crate = sails::scale_info)]
enum DogEvents {
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

// Implementing `AsRef` for each of the extended services
impl AsRef<WalkerService> for DogService {
    fn as_ref(&self) -> &WalkerService {
        &self.walker
    }
}

// Implementing `AsRef` for each of the extended services
impl AsRef<MammalService> for DogService {
    fn as_ref(&self) -> &MammalService {
        &self.mammal
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
#[gservice(extends = [MammalService, WalkerService], events = DogEvents)]
impl DogService {
    pub fn make_sound(&mut self) -> &'static str {
        self.notify_on(DogEvents::Barked).unwrap();
        "Woof! Woof!"
    }
}
