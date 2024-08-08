#![no_std]

#[cfg(feature = "wasm-binary")]
#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

use sails_rs::prelude::*;

struct {{ program-name }}Service(());

#[sails_rs::service]
impl {{ program-name }}Service {
    pub fn new() -> Self {
        Self(())
    }

    pub fn do_something(&mut self) -> String {
        "Hello from {{ program-name }}!".to_string()
    }
}

pub struct {{ program-name }}Program(());

#[sails_rs::program]
impl {{ program-name }}Program {
    pub fn new() -> Self {
        Self(())
    }

    pub fn {{ program-name-snake}}(&self) -> {{ program-name }}Service {
        {{ program-name }}Service::new()
    }
}

#[cfg(feature = "wasm-binary")]
#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
