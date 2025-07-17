#![no_std]

use sails_rs::{builtins::*, calls::Call, gstd::calls::GStdRemoting, prelude::*};

struct ProxyBroker;

#[sails_rs::service]
impl ProxyBroker {
    #[export]
    pub async fn add_proxy(&mut self, delegate: ActorId, proxy_type: ProxyType) -> Vec<u8> {
        let proxy_builtin_client = ProxyBuiltin::new(GStdRemoting::new());

        // todo [sab] error type
        proxy_builtin_client
            .add_proxy(delegate, proxy_type)
            .send_recv(PROXY_BUILTIN_ID)
            .await
            .unwrap_or_else(|e| panic!("failed sending proxy builtin request: {e}"))
    }
}

#[derive(Default)]
pub struct Program(());

#[sails_rs::program]
impl Program {
    // Program's constructor
    pub fn new() -> Self {
        Self(())
    }

    // Exposed service
    pub fn proxy_broker(&self) -> ProxyBroker {
        ProxyBroker
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
