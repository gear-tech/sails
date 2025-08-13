#![no_std]

use demo_client::{this_that::ThisThatImpl, *};
use sails_rename::{ActorId, client::*};

mod this_that;

pub struct ProxyProgram {
    this_that_addr: ActorId,
}

#[sails_rename::program(crate = sails_rename)]
impl ProxyProgram {
    pub fn new(this_that_addr: ActorId) -> Self {
        Self { this_that_addr }
    }

    pub fn this_that_caller(&self) -> this_that::ThisThatCaller<Service<DefaultEnv, ThisThatImpl>> {
        let this_that_client = DemoClientProgram::client(DefaultEnv::default(), self.this_that_addr).this_that();
        this_that::ThisThatCaller::new(this_that_client)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
