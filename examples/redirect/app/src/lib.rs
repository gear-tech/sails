#![no_std]

use sails_rs::{gstd, prelude::*};

struct RedirectService;

#[sails_rs::service]
impl RedirectService {
    pub fn new() -> Self {
        Self
    }

    // Service's method (command)
    pub fn exit(&mut self, inheritor_id: ActorId) {
        let program_id = gstd::exec::program_id();
        if program_id != inheritor_id {
            // panic!("Program ID: {} => {}", program_id, inheritor_id);
            gstd::exec::exit(inheritor_id)
        }
    }

    // Service's query
    pub async fn get_program_id(&self) -> ActorId {
        gstd::exec::program_id()
    }
}

pub struct RedirectProgram(());

#[sails_rs::program]
impl RedirectProgram {
    // Program's constructor
    pub fn new() -> Self {
        Self(())
    }

    // Exposed service
    pub fn redirect(&self) -> RedirectService {
        RedirectService::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
