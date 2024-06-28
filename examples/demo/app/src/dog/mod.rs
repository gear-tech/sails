use crate::mammal::MammalService;
use demo_walker::WalkerService;
use sails_rtl::{gstd::gservice, prelude::*};

#[derive(Encode, TypeInfo)]
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
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

impl AsRef<WalkerService> for DogService {
    fn as_ref(&self) -> &WalkerService {
        &self.walker
    }
}

impl AsRef<MammalService> for DogService {
    fn as_ref(&self) -> &MammalService {
        &self.mammal
    }
}

#[gservice(extends = [MammalService, WalkerService], events = DogEvents)]
impl DogService {
    pub fn make_sound(&mut self) -> &'static str {
        self.notify_on(DogEvents::Barked).unwrap();
        "Woof! Woof!"
    }
}
