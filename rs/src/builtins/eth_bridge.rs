use crate::{
    builtins::{BuiltinsRemoting, builtin_action},
    calls::{ActionIo, Call, RemotingAction},
    errors::{Error, Result},
    prelude::{Decode, Encode, Vec},
};
pub use gbuiltin_eth_bridge::{Request as EthBridgeRequest, Response as EthBridgeResponse};
use gprimitives::H160;

builtin_action!(
    EthBridgeRequest,
    EthBridgeBuiltin,
    SendEthMessage { destination: H160, payload: Vec<u8> } => EthBridgeResponse
);

pub struct EthBridgeBuiltin<R> {
    remoting: R,
}

impl<R: BuiltinsRemoting + Clone> EthBridgeBuiltin<R> {
    pub fn new(remoting: R) -> Self {
        Self { remoting }
    }
}

pub trait EthBridgeBuiltinTrait {
    type Args;

    /// Sends an Ethereum message to the specified destination on the Ethereum network with the given payload.
    fn send_eth_message(
        &self,
        destination: H160,
        payload: Vec<u8>,
    ) -> impl Call<Output = EthBridgeResponse, Args = Self::Args>;
}

impl<R: BuiltinsRemoting + Clone> EthBridgeBuiltinTrait for EthBridgeBuiltin<R> {
    type Args = R::Args;

    fn send_eth_message(
        &self,
        destination: H160,
        payload: Vec<u8>,
    ) -> impl Call<Output = EthBridgeResponse, Args = R::Args> {
        self.send_eth_message(destination, payload)
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
