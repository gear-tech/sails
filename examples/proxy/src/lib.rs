#![no_std]

use demo_client::ThisThat;
use sails_rename::gstd::{calls::GStdRemoting, program};

mod this_that;

#[derive(Default)]
pub struct ProxyProgram(());

#[program(crate = sails_rename)]
impl ProxyProgram {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn this_that_caller(&self) -> this_that::ThisThatCaller<ThisThat<GStdRemoting>> {
        let this_that_client = ThisThat::new(GStdRemoting);
        this_that::ThisThatCaller::new(this_that_client)
    }
}
