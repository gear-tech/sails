#![no_std]
#![allow(unexpected_cfgs)]

extern crate alloc;

use services::Catalog;

// Exposed publicly because of tests which use generated data
// while there is no generated client
pub mod services;

#[derive(Default)]
pub struct Program;

#[sails_rs::program]
impl Program {
    // Initialize program and seed hosted services
    pub fn new() -> Self {
        Catalog::seed();
        Self
    }

    // Expose hosted service
    #[sails_rs::export(route = "RmrkCatalog")]
    pub fn catalog(&self) -> Catalog {
        Catalog
    }
}
