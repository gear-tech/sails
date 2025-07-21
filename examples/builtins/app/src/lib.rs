#![no_std]

mod bls381;
mod eth_bridge;
mod proxy;
mod staking;

use bls381::Bls381Broker;
use eth_bridge::EthBridgeBroker;
use proxy::ProxyBroker;
use sails_rs::{builtins::*, calls::Call, gstd::calls::GStdRemoting, prelude::*};
use staking::StakingBroker;

#[derive(Default)]
pub struct BuiltinsBroker(());

#[sails_rs::program]
impl BuiltinsBroker {
    // Program's constructor
    pub fn new() -> Self {
        Self(())
    }

    pub fn proxy_broker(&self) -> ProxyBroker {
        ProxyBroker
    }

    pub fn staking_broker(&self) -> StakingBroker {
        StakingBroker
    }

    pub fn bls381_broker(&self) -> Bls381Broker {
        Bls381Broker
    }

    pub fn eth_bridge_broker(&self) -> EthBridgeBroker {
        EthBridgeBroker
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
