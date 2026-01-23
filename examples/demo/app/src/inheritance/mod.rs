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

    // OVERRIDE 2: From Mammal (Short Typed Style)
    // Note: unwrap_result is taken from #[export] if needed
    #[export(overrides = MammalService, unwrap_result)]
    pub fn make_sound(&mut self) -> Result<&'static str, String> {
        Ok("Inherited Sound")
    }

    // OVERRIDE 3: From Mammal (Manual Style)
    // AvgWeight is a Query. Commands are 0..N, Queries are N..M.
    // Mammal has: MakeSound (Cmd, 0). Sleep (Cmd, 1). AvgWeight (Query, 2).
    #[export(overrides = MammalService, entry_id = 2)]
    pub fn avg_weight(&self) -> u32 {
        1000 // Custom weight
    }

    // OVERRIDE 4: From Mammal (Synchronicity must match)
    #[export(overrides = MammalService)]
    pub async fn sleep(&mut self) -> String {
        "Awake!".to_string()
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
    #[export(overrides = InheritanceService)]
    pub fn walk(&mut self, _dx: i32, _dy: i32) {
        // Even more custom logic
    }

    // OVERRIDE 2: From Inheritance (which was overridden from Mammal)
    #[export(overrides = InheritanceService)]
    pub fn make_sound(&mut self) -> &'static str {
        "Chain Sound"
    }

    // OVERRIDE 3: From original Walker (via parent)
    // NOTE: We must target WalkerService directly because InheritanceService does not override 'position',
    // so it's not present in InheritanceService::METHODS. Our validation is non-recursive
    #[export(overrides = WalkerService)]
    pub fn position(&self) -> (i32, i32) {
        (99, 99) // Fixed position
    }
}
