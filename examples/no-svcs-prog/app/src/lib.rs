#![no_std]

use sails_macros::gprogram;
use sails_rtl::prelude::*;

#[derive(Default)]
pub struct Program;

#[gprogram]
impl Program {
    pub fn new() -> Self {
        Self
    }
}
