#![no_std]

use services::Catalog;

// Exposed publicly because of tests which use generated data
// while there is no generated client
pub mod services;

#[derive(Default)]
pub struct Program;

#[sails::program]
impl Program {
    // Initialize program and seed hosted services
    pub fn new() -> Self {
        Catalog::seed();
        Self
    }

    // Expose hosted service
    #[sails::export(route = "RmrkCatalog")]
    pub fn catalog(&self) -> Catalog {
        Catalog
    }
}
