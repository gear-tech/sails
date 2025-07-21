use crate::{
    ActorId,
    builtins::{BuiltinsRemoting, builtin_action},
    calls::{ActionIo, Call, RemotingAction},
    errors::{Error, Result},
    prelude::{Decode, Encode, Vec},
};
pub use gbuiltin_eth_bridge::{Request as EthBridgeRequest, Response as EthBridgeResponse};
use gprimitives::H160;

/// Gear protocol eth-bridge builtin id is 0xf2816ced0b15749595392d3a18b5a2363d6fefe5b3b6153739f218151b7acdbf
pub const ETH_BRIDGE_BUILTIN_ID: ActorId = ActorId::new([
    0xf2, 0x81, 0x6c, 0xed, 0x0b, 0x15, 0x74, 0x95, 0x95, 0x39, 0x2d, 0x3a, 0x18, 0xb5, 0xa2, 0x36,
    0x3d, 0x6f, 0xef, 0xe5, 0xb3, 0xb6, 0x15, 0x37, 0x39, 0xf2, 0x18, 0x15, 0x1b, 0x7a, 0xcd, 0xbf,
]);

builtin_action!(
    EthBridgeRequest,
    EthBridgeRemoting,
    SendEthMessage { destination: H160, payload: Vec<u8> } => EthBridgeResponse
);

pub struct EthBridgeRemoting<R> {
    remoting: R,
}

impl<R: BuiltinsRemoting + Clone> EthBridgeRemoting<R> {
    pub fn new(remoting: R) -> Self {
        Self { remoting }
    }
}

#[test]
fn test_id() {
    let expected = hex::decode("f2816ced0b15749595392d3a18b5a2363d6fefe5b3b6153739f218151b7acdbf")
        .expect("Failed to decode hex");
    assert_eq!(ETH_BRIDGE_BUILTIN_ID.into_bytes().to_vec(), expected);
}
