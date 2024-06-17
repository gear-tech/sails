#![no_std]

use sails_rtl::gstd::gprogram;

pub mod service;

#[derive(Default)]
pub struct Program;

#[gprogram]
impl Program {
    pub fn new() -> Self {
        Self
    }

    pub fn service(&self) -> service::MyService {
        service::MyService::new()
    }
}
