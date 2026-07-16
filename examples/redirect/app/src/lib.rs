#![no_std]

use sails::{gstd, prelude::*};

#[derive(Default)]
struct RedirectService;

impl RedirectService {
    pub const fn new() -> Self {
        Self
    }
}

#[sails::service]
impl RedirectService {
    /// Exit from program with inheritor ID
    #[sails::export]
    pub fn exit(&mut self, inheritor_id: ActorId) {
        let program_id = gstd::exec::program_id();
        if program_id != inheritor_id {
            gstd::exec::exit(inheritor_id)
        }
    }

    /// Returns program ID of the current program
    #[sails::export]
    pub async fn get_program_id(&self) -> ActorId {
        gstd::exec::program_id()
    }
}

#[derive(Default)]
pub struct RedirectProgram;

#[sails::program]
impl RedirectProgram {
    // Redirect Program's constructor
    pub fn new() -> Self {
        Self
    }

    // Exposed Redirect service
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
