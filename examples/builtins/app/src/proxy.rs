use super::*;

pub struct ProxyBroker;

#[sails_rs::service]
impl ProxyBroker {
    #[export]
    pub async fn add_proxy(
        &mut self,
        delegate: ActorId,
        proxy_type: ProxyType,
    ) -> Result<Vec<u8>, String> {
        let proxy_builtin_client = ProxyBuiltin::new(GStdRemoting::new());

        proxy_builtin_client
            .add_proxy(delegate, proxy_type)
            .send_recv(PROXY_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending proxy builtin request: {e}"))
    }

    #[export]
    pub async fn remove_proxy(
        &mut self,
        delegate: ActorId,
        proxy_type: ProxyType,
    ) -> Result<Vec<u8>, String> {
        let proxy_builtin_client = ProxyBuiltin::new(GStdRemoting::new());

        proxy_builtin_client
            .remove_proxy(delegate, proxy_type)
            .send_recv(PROXY_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending proxy builtin request: {e}"))
    }
}
