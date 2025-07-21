use super::*;

pub struct EthBridgeBroker;

#[sails_rs::service]
impl EthBridgeBroker {
    #[export]
    pub async fn send_eth_message(
        &mut self,
        destination: H160,
        payload: Vec<u8>,
    ) -> Result<EthBridgeResponse, String> {
        let eth_bridge_builtin_client = EthBridgeRemoting::new(GStdRemoting::new());

        eth_bridge_builtin_client
            .send_eth_message(destination, payload)
            .send_recv(ETH_BRIDGE_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending eth bridge builtin request: {e}"))
    }
}
