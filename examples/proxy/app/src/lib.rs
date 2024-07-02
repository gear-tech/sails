#![no_std]

use demo_client::ThisThat;
use sails_rtl::gstd::{
    calls::{GStdArgs, GStdRemoting},
    gprogram,
};

mod this_that;

#[derive(Default)]
pub struct ProxyProgram(());

#[gprogram]
impl ProxyProgram {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn this_that_caller(
        &self,
    ) -> this_that::ThisThatCaller<ThisThat<GStdRemoting, GStdArgs>, GStdArgs> {
        let this_that_client = ThisThat::new(GStdRemoting);
        this_that::ThisThatCaller::new(this_that_client)
    }
}
