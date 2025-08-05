#![no_std]

pub mod client {
    include!("./ping_pong_client.rs");
}

use client::{
    PingPongPayload as PingPongPayloadC, PingPongService as PingPongServiceC,
    traits::PingPongService as _,
};
use sails_rs::{calls::Call, gstd::calls::GStdRemoting, prelude::*};

#[derive(Debug, Clone, Copy, Decode, Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum PingPongPayload {
    Start(ActorId),
    Ping,
    Pong,
    Finished,
}

pub struct PingPongService;

#[sails_rs::service]
impl PingPongService {
    #[export]
    pub async fn ping(&mut self, payload: PingPongPayload) -> PingPongPayload {
        match payload {
            PingPongPayload::Start(actor_id) => {
                let mut api = PingPongServiceC::new(GStdRemoting::new());
                let result = api
                    .ping(PingPongPayloadC::Ping)
                    .send_recv(actor_id)
                    .await
                    .unwrap_or_else(|e| {
                        panic!("Failed to receiving successful ping result: {e:?}")
                    });

                if matches!(result, PingPongPayloadC::Pong) {
                    PingPongPayload::Finished
                } else {
                    panic!("Unexpected payload received: {result:?}")
                }
            }
            PingPongPayload::Ping => PingPongPayload::Pong,
            PingPongPayload::Pong | PingPongPayload::Finished => {
                // Respond with Finished
                unreachable!("Unexpected payload received: {payload:?}")
            }
        }
    }
}

pub struct PingPongProgram;

#[sails_rs::program]
impl PingPongProgram {
    #[allow(clippy::new_without_default)]
    pub fn new_for_bench() -> Self {
        Self
    }

    pub fn ping_pong_service(&self) -> PingPongService {
        PingPongService
    }
}
