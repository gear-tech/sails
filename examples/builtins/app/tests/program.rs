/// These tests make interact with builtins from app context.
use builtins_example_app::WASM_BINARY;
use builtins_example_client::{
    BuiltinsExampleClientFactory, ProxyBroker, ProxyType,
    traits::{BuiltinsExampleClientFactory as _, ProxyBroker as _},
};
use gclient::GearApi;
use sails_rs::{
    H256,
    calls::{Activation, Call},
    gclient::calls::GClientRemoting,
};

#[tokio::test]
async fn call_proxy_builtin_from_app() {
    let api = GearApi::dev().await.unwrap();

    let (code_id, _) = api
        .upload_code(WASM_BINARY)
        .await
        .expect("Failed to upload program code");

    let remoting = GClientRemoting::new(api.clone());

    let builtins_broker_pid = BuiltinsExampleClientFactory::new(remoting.clone())
        .new()
        .send_recv(code_id, b"builtins-example-app")
        .await
        .expect("Failed program init message");

    api.transfer_keep_alive(builtins_broker_pid, 100_000_000_000_000_000_000)
        .await
        .expect("Failed to transfer funds to program");

    let mut proxy_broker_client = ProxyBroker::new(remoting.clone());
    let resp = proxy_broker_client
        .add_proxy(H256::random().into(), ProxyType::Any)
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send proxy request");

    assert_eq!(resp, Ok(Vec::<u8>::new()));
}
