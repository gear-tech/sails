#![no_std]

use sails_rtl::gstd::{gprogram, groute};
use service::PingService;

pub mod service;

#[derive(Default)]
pub struct Program;

#[gprogram]
impl Program {
    pub fn new() -> Self {
        Self
    }

    #[groute("Ping")]
    pub fn ping(&self) -> service::PingService {
        PingService::new()
    }
}
