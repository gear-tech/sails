#![no_std]

use demo_client::{DemoClient, DemoClientProgram, counter::Counter};
use futures::{future, FutureExt};
use redirect_client::{redirect::Redirect as _, *};
use sails_rs::{client::*, prelude::*};

pub struct AggregatorService {
    counter: Service<demo_client::counter::CounterImpl>,
}

impl AggregatorService {
    pub fn new(target: ActorId) -> Self {
        let demo = DemoClientProgram::client(target);
        Self {
            counter: demo.counter(),
        }
    }
}

#[sails_rs::service]
impl AggregatorService {
    /// Fetches the counter value.
    #[export]
    pub async fn fetch_value(&self) -> u32 {
        self.counter.value().send_for_reply().expect("Send failed").await.expect("Await failed")
    }

    /// Fetches two values concurrently using `future::join`.
    #[export]
    pub async fn fetch_summary(&self) -> (u32, u32) {
        let call1 = self.counter.value().send_for_reply().expect("First send failed");
        let call2 = self.counter.value().send_for_reply().expect("Second send failed");

        let (res1, res2) = future::join(call1, call2).await;
        
        let val1: u32 = res1.expect("First await failed");
        let val2: u32 = res2.expect("Second await failed");
        (val1, val2)
    }

    /// Demonstrates `future::select` between a real call and a fallback.
    #[export]
    pub async fn fetch_with_fallback(&self, use_fallback: bool) -> u32 {
        let call = self.counter.value().send_for_reply().expect("Send failed");
        
        if use_fallback {
            let fallback = future::ready(Ok::<u32, sails_rs::errors::Error>(999));
            match future::select(call, fallback.boxed()).await {
                future::Either::Left((res, _)) => res.expect("Call failed"),
                future::Either::Right((res, _)) => res.expect("Fallback failed"),
            }
        } else {
            call.await.expect("Await failed")
        }
    }

    /// Races two real calls and returns the first one.
    #[export]
    pub async fn fetch_fastest(&self, target1: ActorId, target2: ActorId) -> u32 {
        let call1 = DemoClientProgram::client(target1).counter().value().send_for_reply().expect("Send 1 failed");
        let call2 = DemoClientProgram::client(target2).counter().value().send_for_reply().expect("Send 2 failed");

        match future::select(call1.fuse(), call2.fuse()).await {
            future::Either::Left((res, _)) => res.expect("Call 1 failed"),
            future::Either::Right((res, _)) => res.expect("Call 2 failed"),
        }
    }

    /// Fetches target's program ID with redirection enabled.
    #[export]
    pub async fn fetch_redirect_id(&self, target: ActorId) -> ActorId {
        let redirect_program = RedirectClientProgram::client(target);
        redirect_program.redirect()
            .get_program_id()
            .with_redirect_on_exit(true)
            .send_for_reply()
            .expect("Send failed")
            .await
            .expect("Await failed")
    }

    /// Intentionally panics by calling send_for_reply twice.
    #[export]
    pub async fn test_poll_after_completion(&self) {
        let call = self.counter.value().send_for_reply().expect("First send failed");
        let _ = call.send_for_reply().expect("Second send failed");
    }

    /// Fetches value from a specific address, handling potential errors.
    #[export]
    pub async fn fetch_from_address(&self, target: ActorId) -> Result<u32, String> {
        let demo = DemoClientProgram::client(target);
        let call = demo.counter().value().send_for_reply().map_err(|e| e.to_string())?;
        call.await.map_err(|e| e.to_string())
    }
}

pub struct AggregatorProgram {
    target: ActorId,
}

#[sails_rs::program]
impl AggregatorProgram {
    pub fn new(target: ActorId) -> Self {
        Self { target }
    }

    pub fn aggregator(&self) -> AggregatorService {
        AggregatorService::new(self.target)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
