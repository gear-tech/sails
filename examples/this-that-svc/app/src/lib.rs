#![no_std]

use sails_rtl::gstd::gprogram;

pub mod this_that_svc;

#[derive(Default)]
pub struct Program;

#[gprogram]
impl Program {
    pub fn new() -> Self {
        Self
    }

    pub fn this_that_svc(&self) -> this_that_svc::ThisThatSvc {
        this_that_svc::ThisThatSvc::new()
    }
}
