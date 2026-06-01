#![no_std]

use sails_rs::prelude::*;

struct NoopBaseline;

#[sails_rs::service]
impl NoopBaseline {
    #[export]
    pub fn do_nothing(&mut self) {}
}

#[derive(Default)]
pub struct Program;

#[sails_rs::program]
impl Program {
    // Exposed service
    pub fn noop_baseline(&self) -> NoopBaseline {
        NoopBaseline
    }
}
