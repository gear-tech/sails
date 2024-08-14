#![no_std]

#[cfg(feature = "wasm-binary")]
#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

use sails_rs::prelude::*;

struct {{ service-struct-name }}(());

#[sails_rs::service]
impl {{ service-struct-name }} {
    pub fn new() -> Self {
        Self(())
    }

    // Service's method (command)
    pub fn do_something(&mut self) -> String {
        "Hello from {{ service-name }}!".to_string()
    }
}

pub struct {{ program-struct-name }}(());

#[sails_rs::program]
impl {{ program-struct-name }} {
    // Program's constructor
    pub fn new() -> Self {
        Self(())
    }

    // Exposed service
    pub fn {{ service-name-snake}}(&self) -> {{ service-struct-name }} {
        {{ service-struct-name }}::new()
    }
}

#[cfg(feature = "wasm-binary")]
#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
