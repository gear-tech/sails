use gclient::GearApi;
/// These tests make interact with builtins from non-app (program) context via extrinsics (calls).
use sails_rs::{
    H256,
    builtins::{PROXY_BUILTIN_ID, ProxyBuiltin, ProxyType as SailsProxyType},
    calls::Call,
    gclient::calls::GClientRemoting,
};

#[tokio::test]
async fn call_proxy_builtin_with_extrinsic() {
    let api = GearApi::dev().await.unwrap();

    let remoting = GClientRemoting::new(api.clone());

    let proxy = ProxyBuiltin::new(remoting);
    let random_actor_id = H256::random().into();
    let res = proxy
        .add_proxy(random_actor_id, SailsProxyType::Any)
        .send_recv(PROXY_BUILTIN_ID)
        .await
        .expect("Failed to send proxy request");

    assert_eq!(res, Vec::<u8>::new());

    let res = proxy
        .remove_proxy(random_actor_id, SailsProxyType::Any)
        .send_recv(PROXY_BUILTIN_ID)
        .await
        .expect("Failed to send proxy request");

    assert_eq!(res, Vec::<u8>::new());
}
