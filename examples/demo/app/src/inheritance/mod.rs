use crate::dog::DogService;
use crate::mammal::MammalService;
use demo_walker::WalkerService;
use sails_rs::prelude::*;

pub struct ChainService {
    dog: DogService,
}

impl ChainService {
    pub fn new(dog: DogService) -> Self {
        Self { dog }
    }
}

impl From<ChainService> for DogService {
    fn from(value: ChainService) -> Self {
        value.dog
    }
}

#[service(extends = DogService)]
impl ChainService {
    // 0. Shadowing: Same name as in DogService, but NOT an override.
    #[export]
    pub fn make_sound(&mut self) -> &'static str {
        "Chain Woof!"
    }

    // 1. By Entry ID: Name is resolved from base MammalService metadata (ID 0 is MakeSound)
    #[export(overrides = MammalService, entry_id = 0)]
    pub fn mammal_make_sound(&mut self) -> &'static str {
        "Chain Mammal Sound (via ID)"
    }

    // 2. By Explicit Route: Original name "Walk" is provided manually
    #[export(overrides = WalkerService, route = "Walk")]
    pub fn walker_walk(&mut self, _dx: i32, _dy: i32) {
        // Chain walks differently
    }

    // 3. By Function Name (Default): "avg_weight" matches in both services
    #[export(overrides = MammalService)]
    pub fn avg_weight(&self) -> u32 {
        99 // Chain weight
    }
}
