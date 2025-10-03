#![no_std]

use redirect_client::{redirect::Redirect as _, *};
use sails_rs::{client::*, prelude::*};

struct ProxyService(Service<redirect_client::redirect::RedirectImpl>);

impl ProxyService {
    pub fn new(target: ActorId) -> Self {
        Self(RedirectClientProgram::client(target).redirect())
    }
}

#[sails_rs::service]
impl ProxyService {
    /// Get program ID of the target program via client
    #[sails_rs::export]
    pub async fn get_program_id(&self) -> ActorId {
        self.0
            .get_program_id()
            // Set flag to redirect on exit
            .with_redirect_on_exit(true)
            .await
            .unwrap()
    }
}

pub struct ProxyProgram(ActorId);

#[sails_rs::program]
impl ProxyProgram {
    /// Proxy Program's constructor
    pub fn new(target: ActorId) -> Self {
        Self(target)
    }

    /// Exposed Proxy Service
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
