#![no_std]

use sails_rtl::gstd::gprogram;

#[derive(Default)]
pub struct Program;

#[gprogram]
impl Program {
    pub fn new() -> Self {
        Self
    }
}
