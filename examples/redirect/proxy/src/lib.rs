#![no_std]

use redirect_client::{Redirect, traits::Redirect as _};
use sails_rs::{
    calls::{Action, Query as _},
    gstd::calls::*,
    prelude::*,
};

struct ProxyService(ActorId);

#[sails_rs::service]
impl ProxyService {
    pub fn new(target: ActorId) -> Self {
        Self(target)
    }

    // Service's query
    pub async fn get_program_id(&self) -> ActorId {
        let client = Redirect::new(GStdRemoting::new());
        client
            .get_program_id()
            .with_redirect_on_exit(true)
            .recv(self.0)
            .await
            .unwrap()
    }
}

pub struct ProxyProgram(ActorId);

#[sails_rs::program]
impl ProxyProgram {
    // Program's constructor
    pub fn new(target: ActorId) -> Self {
        Self(target)
    }

    // Exposed service
    pub fn proxy(&self) -> ProxyService {
        ProxyService::new(self.0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
