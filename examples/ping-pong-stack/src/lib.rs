#![no_std]

use client::{
    PingPongStack as _, PingPongStackCtors as _, PingPongStackProgram,
    ping_pong_stack::PingPongStack as _,
};
use sails_rs::{client::Program as _, gstd::*, prelude::*};

struct PingPongStack(ActorId);

impl PingPongStack {
    pub fn new(actor_id: ActorId) -> Self {
        Self(actor_id)
    }
}

#[sails_rs::service]
impl PingPongStack {
    #[export]
    pub async fn start(&mut self, limit: u32) {
        if self.0 == ActorId::zero() {
            panic!("Pong actor not set")
        }
        self.call(self.0, limit).await;
    }

    #[export]
    pub async fn ping(&mut self, countdown: u32) {
        let source = Syscall::message_source();
        self.call(source, countdown - 1).await;
    }

    #[inline]
    async fn call(&mut self, actor_id: ActorId, countdown: u32) -> bool {
        sails_rs::gstd::debug!("Ping: {countdown}, actor_id: {actor_id}");
        if countdown > 0 {
            let mut api = PingPongStackProgram::client(actor_id).ping_pong_stack();
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
pub struct Program(ActorId);

#[sails_rs::program]
impl Program {
    #[export(payable)]
    pub async fn create_ping(code_id: CodeId) -> Self {
        let msg_id = Syscall::message_id();
        let actor = PingPongStackProgram::deploy(code_id, msg_id.into_bytes().into())
            .create_pong()
            .with_reply_deposit(10_000_000_000)
            .await
            .unwrap();
        Self(actor.id())
    }

    pub fn create_pong() -> Self {
        Self(ActorId::zero())
    }

    // Exposed service
    pub fn ping_pong_stack(&self) -> PingPongStack {
        PingPongStack::new(self.0)
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
