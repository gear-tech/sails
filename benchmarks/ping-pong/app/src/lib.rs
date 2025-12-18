#![no_std]

pub mod client {
    include!("./ping_pong_client.rs");
}

use client::{PingPong as _, ping_pong_service::PingPongService as _};
use sails_rs::{client::*, prelude::*};

#[derive(Debug, Clone, Copy, Decode, Encode, TypeInfo, ReflectHash)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
#[reflect_hash(crate = sails_rs)]
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
                let mut api = client::PingPongProgram::client(actor_id).ping_pong_service();
                let result = api
                    .ping(client::PingPongPayload::Ping)
                    .await
                    .unwrap_or_else(|e| {
                        panic!("Failed to receiving successful ping result: {e:?}")
                    });

                if matches!(result, client::PingPongPayload::Pong) {
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
