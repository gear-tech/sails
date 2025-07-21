/// These tests make interact with builtins from app context.
use builtins_example_app::WASM_BINARY;
use builtins_example_client::{
    BuiltinsExampleClientFactory, ProxyBroker, ProxyType, RewardAccount, StakingBroker,
    traits::{BuiltinsExampleClientFactory as _, ProxyBroker as _, StakingBroker as _},
};
use gclient::GearApi;
use sails_rs::{
    H256,
    calls::{Activation, Call},
    gclient::calls::GClientRemoting,
};

const ONE_VARA: u128 = 1_000_000_000_000;

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

    api.transfer_keep_alive(builtins_broker_pid, 10_000 * ONE_VARA)
        .await
        .expect("Failed to transfer funds to program");

    let proxy_addr = H256::random().into();
    let mut proxy_broker_client = ProxyBroker::new(remoting.clone());
    let resp = proxy_broker_client
        .add_proxy(proxy_addr, ProxyType::Any)
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send proxy request");

    assert_eq!(resp, Ok(Vec::<u8>::new()));

    let resp = proxy_broker_client
        .remove_proxy(proxy_addr, ProxyType::Any)
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send proxy request");

    assert_eq!(resp, Ok(Vec::<u8>::new()));
}

#[tokio::test]
async fn call_staking_builtin_from_app() {
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

    api.transfer_keep_alive(builtins_broker_pid, 10_000 * ONE_VARA)
        .await
        .expect("Failed to transfer funds to program");

    let mut staking_builtin_client = StakingBroker::new(remoting.clone());

    let resp = staking_builtin_client
        .bond(5 * ONE_VARA, RewardAccount::None)
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send bond request");
    assert_eq!(resp, Ok(Vec::<u8>::new()));

    let resp = staking_builtin_client
        .bond_extra(2 * ONE_VARA)
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send bond_extra request");
    assert_eq!(resp, Ok(Vec::<u8>::new()));

    let resp = staking_builtin_client
        .rebond(4 * ONE_VARA)
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send rebond request")
        .unwrap_err();

    assert!(resp.to_string().contains("NoUnlockChunk"));

    let resp = staking_builtin_client
        .set_payee(RewardAccount::Program)
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send set_payee request");

    assert_eq!(resp, Ok(Vec::<u8>::new()));

    // todo [sab] invalid gas count for `unbond` call

    let resp = staking_builtin_client
        .chill()
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send chill request");

    assert_eq!(resp, Ok(Vec::<u8>::new()));

    // todo [sab] invalid gas count for `withdraw_unbonded` call

    let resp = staking_builtin_client
        .nominate(vec![H256::random().into()])
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send nominate request")
        .unwrap_err();

    assert!(resp.to_string().contains("InsufficientBond"));

    // todo [sab] invalid gas count for `payout_stakers` call
}
