#![no_std]

use sails_rs::prelude::*;

pub struct NoopSailsService;

#[sails_rs::service]
impl NoopSailsService {
    #[export]
    pub fn noop(&mut self) -> bool {
        true
    }
}

pub struct NoopSailsProgram;

#[sails_rs::program]
impl NoopSailsProgram {
    pub fn new_for_bench() -> Self {
        Self
    }

    pub fn noop_sails(&self) -> NoopSailsService {
        NoopSailsService
    }
}
