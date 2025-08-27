#![no_std]

#[cfg(feature = "mockall")]
#[cfg(not(target_arch = "wasm32"))]
pub extern crate std;

use crate::catalogs::{RmrkCatalog as _, RmrkCatalogProgram, rmrk_catalog::RmrkCatalogImpl};
use sails_rs::{
    client::{Program as _, *},
    prelude::*,
};
use services::ResourceStorage;

mod catalogs;
// Exposed publicly because of tests which use generated data
// while there is no generated client
pub mod services;

#[derive(Default)]
pub struct Program;

#[program]
impl Program {
    // Initialize program and seed hosted services
    pub fn new() -> Self {
        ResourceStorage::<Service<DefaultEnv, RmrkCatalogImpl>>::seed();
        Self
    }

    // Expose hosted service
    #[export(route = "RmrkResource")]
    pub fn resource_storage(&self) -> ResourceStorage<Service<DefaultEnv, RmrkCatalogImpl>> {
        let rmrk_catalog_client =
            RmrkCatalogProgram::client(DefaultEnv::default(), ActorId::zero()).rmrk_catalog();
        ResourceStorage::new(rmrk_catalog_client)
    }
}
