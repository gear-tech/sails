use crate::mammal::{MammalService, mammal_service_methods};
use demo_walker::{WalkerService, walker_service_methods};
use sails_rs::prelude::*;

// 1. Composite Style Service
#[derive(Clone)]
pub struct InheritanceService {
    walker: WalkerService,
    mammal: MammalService,
}

impl InheritanceService {
    pub fn new(walker: WalkerService, mammal: MammalService) -> Self {
        Self { walker, mammal }
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
    #[override_entry(walker_service_methods::Walk)]
    pub fn walk(&mut self, _dx: i32, _dy: i32) {
        // Custom logic: do nothing or something else
    }

    // OVERRIDE 2: From Mammal (Short Typed Style)
    // Note: unwrap_result is taken from #[export] if needed
    #[override_entry(mammal_service_methods::MakeSound)]
    #[export(unwrap_result)]
    pub fn make_sound(&mut self) -> Result<&'static str, String> {
        Ok("Inherited Sound")
    }

    // OVERRIDE 3: From Mammal (Manual Style)
    // AvgWeight is a Query. Commands are 0..N, Queries are N..M.
    // Mammal has: MakeSound (Cmd, 0). AvgWeight (Query, 1).
    #[override_entry(MammalService, 1)]
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
    // OVERRIDE 1: From Inheritance (which was already overridden from Walker)
    #[override_entry(inheritance_service_methods::Walk)]
    pub fn walk(&mut self, _dx: i32, _dy: i32) {
        // Even more custom logic
    }

    // OVERRIDE 2: From Inheritance (which was overridden from Mammal)
    #[override_entry(inheritance_service_methods::MakeSound)]
    pub fn make_sound(&mut self) -> &'static str {
        "Chain Sound"
    }

    // OVERRIDE 3: From original Walker (via parent)
    #[override_entry(walker_service_methods::Position)]
    pub fn position(&self) -> (i32, i32) {
        (99, 99) // Fixed position
    }
}
