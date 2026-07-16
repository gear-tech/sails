#![no_std]

use sails::prelude::*;

struct NoopBaseline;

#[sails::service]
impl NoopBaseline {
    #[export]
    pub fn do_nothing(&mut self) {}
}

#[derive(Default)]
pub struct Program;

#[sails::program]
impl Program {
    // Exposed service
    pub fn noop_baseline(&self) -> NoopBaseline {
        NoopBaseline
    }
}
