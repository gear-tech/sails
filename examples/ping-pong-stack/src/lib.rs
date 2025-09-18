#![no_std]

use client::{PingPongStack as _, ping_pong_stack::PingPongStack as _};
use sails_rs::{client::Program as _, gstd::*, prelude::*};

struct PingPongStack;

impl PingPongStack {
    pub fn new() -> Self {
        Self
    }
}

#[sails_rs::service]
impl PingPongStack {
    #[export]
    pub async fn start(&mut self, actor_id: ActorId, limit: u32) {
        self.call(actor_id, limit).await;
    }

    #[export]
    pub async fn ping(&mut self, countdown: u32) {
        let source = msg::source();
        self.call(source, countdown - 1).await;
    }

    #[inline]
    async fn call(&mut self, actor_id: ActorId, countdown: u32) -> bool {
        sails_rs::gstd::debug!("Ping: {countdown}, actor_id: {actor_id}");
        if countdown > 0 {
            let mut api = client::PingPongStackProgram::client(actor_id).ping_pong_stack();
            let _res = api.ping(countdown).with_reply_deposit(10_000_000_000).await;
            sails_rs::gstd::debug!("Result: {_res:?}");
            debug_assert!(_res.is_ok());
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct Program;

#[sails_rs::program]
impl Program {
    pub fn new_for_bench() -> Self {
        Self
    }

    // Exposed service
    pub fn ping_pong_stack(&self) -> PingPongStack {
        PingPongStack::new()
    }
}

pub mod client {
    include!("./ping_pong_stack.rs");
}

#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
