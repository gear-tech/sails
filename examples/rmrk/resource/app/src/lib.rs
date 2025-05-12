#![no_std]

#[cfg(feature = "mockall")]
#[cfg(not(target_arch = "wasm32"))]
pub extern crate std;

use sails_rs::gstd::{calls::GStdRemoting, program};
use services::ResourceStorage;

mod catalogs;
// Exposed publicly because of tests which use generated data
// while there is no generated client
pub mod services;

type RmrkCatalog = catalogs::RmrkCatalog<GStdRemoting>;

#[derive(Default)]
pub struct Program;

#[program]
impl Program {
    // Initialize program and seed hosted services
    pub fn new() -> Self {
        ResourceStorage::<RmrkCatalog>::seed();
        Self
    }

    // Expose hosted service
    #[export(route = "RmrkResource")]
    pub fn resource_storage(&self) -> ResourceStorage<RmrkCatalog> {
        ResourceStorage::new(RmrkCatalog::new(GStdRemoting))
    }
}
