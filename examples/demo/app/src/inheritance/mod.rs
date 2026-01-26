use crate::mammal::MammalService;
use demo_walker::WalkerService;
use sails_rs::prelude::*;

// 1. Composite Style Service
#[derive(Clone)]
pub struct InheritanceService {
    walker: WalkerService,
    mammal: MammalService,
}

impl InheritanceService {
    pub fn new(walker: WalkerService) -> Self {
        Self {
            walker,
            mammal: MammalService::new(42),
        }
    }
}

impl From<InheritanceService> for WalkerService {
    fn from(value: InheritanceService) -> Self {
        value.walker
    }
}

impl From<InheritanceService> for MammalService {
    fn from(value: InheritanceService) -> Self {
        value.mammal
    }
}

#[service(extends = [WalkerService, MammalService])]
impl InheritanceService {
    // OVERRIDE 1: From Walker (Short Typed Style)
    #[export(overrides = WalkerService, entry_id = 0)]
    pub fn walk(&mut self, _dx: i32, _dy: i32) {
        // Custom logic: do nothing or something else
    }

    // OVERRIDE 2: From Mammal
    #[export(overrides = MammalService)]
    pub async fn make_sound(&mut self) -> &'static str {
        "Inherited Sound (Async)"
    }

    // OVERRIDE 3: From Mammal (Manual Style)
    // AvgWeight is a Query.
    // Mammal has: MakeSound (Cmd, 0). AvgWeight (Query, 1).
    #[export(overrides = MammalService, entry_id = 1)]
    pub fn avg_weight(&self) -> u32 {
        1000 // Custom weight
    }
}

// 2. Single/Chain Style Service
#[derive(Clone)]
pub struct ChainService {
    parent: InheritanceService,
}

impl ChainService {
    pub fn new(parent: InheritanceService) -> Self {
        Self { parent }
    }
}

impl From<ChainService> for InheritanceService {
    fn from(value: ChainService) -> Self {
        value.parent
    }
}

#[service(extends = InheritanceService)]
impl ChainService {
    // OVERRIDE 2: From Mammal (via parent)
    #[export(overrides = MammalService)]
    pub fn make_sound(&mut self) -> &'static str {
        "Chain Sound"
    }

    // OVERRIDE 3: From original Walker (via parent)
    #[export(overrides = WalkerService)]
    pub fn position(&self) -> (i32, i32) {
        (99, 99) // Fixed position
    }
}
