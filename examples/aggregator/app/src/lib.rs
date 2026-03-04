#![no_std]

use demo_client::{DemoClient, DemoClientProgram, chaos::Chaos, counter::Counter};
use futures::{FutureExt, future};
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
    pub async fn fetch_value(&self) -> Result<u32, String> {
        self.counter
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())
    }

    /// Fetches two values concurrently using `future::join`.
    #[export]
    pub async fn fetch_summary(&self) -> Result<(u32, u32), String> {
        let call1 = self
            .counter
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;
        let call2 = self
            .counter
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;

        let (res1, res2) = future::join(call1, call2).await;

        let val1 = res1.map_err(|e| e.to_string())?;
        let val2 = res2.map_err(|e| e.to_string())?;
        Ok((val1, val2))
    }

    /// Demonstrates `future::select` between a real call and a fallback.
    #[export]
    pub async fn fetch_with_fallback(&self, use_fallback: bool) -> Result<u32, String> {
        let call = self
            .counter
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;

        if use_fallback {
            let fallback = future::ready(Ok::<u32, sails_rs::errors::Error>(999));
            match future::select(call, fallback.boxed()).await {
                future::Either::Left((res, _)) => res.map_err(|e| e.to_string()),
                future::Either::Right((res, _)) => res.map_err(|e| e.to_string()),
            }
        } else {
            call.await.map_err(|e| e.to_string())
        }
    }

    /// Races a slow call (Chaos timeout) and a fast call (Counter).
    /// Returns 1 if Counter wins, 2 if Chaos wins.
    #[export]
    pub async fn fetch_fastest(&self, target: ActorId) -> Result<u32, String> {
        let demo = DemoClientProgram::client(target);

        let slow_call = demo
            .chaos()
            .timeout_wait()
            .send_for_reply()
            .map_err(|e| e.to_string())?;
        let fast_call = demo
            .counter()
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;

        match future::select(slow_call.fuse(), fast_call.fuse()).await {
            future::Either::Left((res, _)) => {
                res.map_err(|e| e.to_string())?;
                Ok(2)
            }
            future::Either::Right((res, _)) => {
                res.map_err(|e| e.to_string())?;
                Ok(1)
            }
        }
    }

    /// Fetches target's program ID with redirection enabled.
    #[export]
    pub async fn fetch_redirect_id(&self, target: ActorId) -> Result<ActorId, String> {
        let redirect_program = RedirectClientProgram::client(target);
        redirect_program
            .redirect()
            .get_program_id()
            .with_redirect_on_exit(true)
            .send_for_reply()
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())
    }

    /// Intentionally panics by calling send_for_reply twice.
    #[export]
    pub async fn test_poll_after_completion(&self) {
        let call = self
            .counter
            .value()
            .send_for_reply()
            .expect("First send failed");
        call.send_for_reply().expect("Second send failed");
    }

    /// Fetches value from a specific address, handling potential errors.
    #[export]
    pub async fn fetch_from_address(&self, target: ActorId) -> Result<u32, String> {
        let demo = DemoClientProgram::client(target);
        let call = demo
            .counter()
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;
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
