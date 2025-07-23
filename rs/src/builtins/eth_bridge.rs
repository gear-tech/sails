use crate::{
    ActorId,
    builtins::{BuiltinsRemoting, builtin_action},
    calls::{ActionIo, Call, RemotingAction},
    errors::{Error, Result},
    prelude::{Decode, Encode, Vec},
};
pub use gbuiltin_eth_bridge::{Request as EthBridgeRequest, Response as EthBridgeResponse};
use gprimitives::H160;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::test_utils::assert_action_codec;
    use crate::prelude::vec;
    use gprimitives::{H256, U256};

    #[test]
    fn test_codec() {
        assert_action_codec!(
            EthBridgeRequest,
            SendEthMessage {
                destination: H160::from([1; 20]),
                payload: vec![1, 2, 3, 4]
            },
            EthBridgeResponse,
            EthMessageQueued {
                nonce: U256::one(),
                hash: H256::from([2; 32])
            }
        );
    }
}
