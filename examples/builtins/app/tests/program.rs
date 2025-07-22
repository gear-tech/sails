/// These tests make interact with builtins from app context.
use builtins_example_app::WASM_BINARY;
use builtins_example_client::{
    traits::{Bls381Broker as _, BuiltinsExampleClientFactory as _, ProxyBroker as _, StakingBroker as _}, Bls381Broker, Bls381Response, BuiltinsExampleClientFactory, ProxyBroker, ProxyType, RewardAccount, StakingBroker
};
use gclient::GearApi;
use sails_rs::{
    H256,
    calls::{Activation, Call},
    gclient::calls::GClientRemoting,
    Encode,
};

const ONE_VARA: u128 = 1_000_000_000_000;

// #[tokio::test]
// async fn call_proxy_builtin_from_app() {
//     let api = GearApi::dev().await.unwrap();

//     let (code_id, _) = api
//         .upload_code(WASM_BINARY)
//         .await
//         .expect("Failed to upload program code");

//     let remoting = GClientRemoting::new(api.clone());

//     let builtins_broker_pid = BuiltinsExampleClientFactory::new(remoting.clone())
//         .new()
//         .send_recv(code_id, b"builtins-example-app")
//         .await
//         .expect("Failed program init message");

//     api.transfer_keep_alive(builtins_broker_pid, 10_000 * ONE_VARA)
//         .await
//         .expect("Failed to transfer funds to program");

//     let proxy_addr = H256::random().into();
//     let mut proxy_broker_client = ProxyBroker::new(remoting.clone());
//     let resp = proxy_broker_client
//         .add_proxy(proxy_addr, ProxyType::Any)
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send proxy request");

//     assert_eq!(resp, Ok(Vec::<u8>::new()));

//     let resp = proxy_broker_client
//         .remove_proxy(proxy_addr, ProxyType::Any)
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send proxy request");

//     assert_eq!(resp, Ok(Vec::<u8>::new()));
// }

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

    // todo [sab] invalid gas count for `unbond` call

    let resp = staking_builtin_client
        .bond(5 * ONE_VARA, RewardAccount::None)
        .send_recv(builtins_broker_pid)
        .await
        .expect("Failed to send bond request");
    assert_eq!(resp, Ok(Vec::<u8>::new()));

    // let resp = staking_builtin_client
    //     .bond_extra(2 * ONE_VARA)
    //     .send_recv(builtins_broker_pid)
    //     .await
    //     .expect("Failed to send bond_extra request");
    // assert_eq!(resp, Ok(Vec::<u8>::new()));

    // let resp = staking_builtin_client
    //     .rebond(4 * ONE_VARA)
    //     .send_recv(builtins_broker_pid)
    //     .await
    //     .expect("Failed to send rebond request")
    //     .unwrap_err();

    // assert!(resp.to_string().contains("NoUnlockChunk"));

    // let resp = staking_builtin_client
    //     .set_payee(RewardAccount::Program)
    //     .send_recv(builtins_broker_pid)
    //     .await
    //     .expect("Failed to send set_payee request");

    // assert_eq!(resp, Ok(Vec::<u8>::new()));

    let resp = staking_builtin_client
        .unbond(2 * ONE_VARA)
        .send_recv(builtins_broker_pid)
        .await;

    println!("Resp: {:#?}", resp);

    // todo [sab] invalid gas count for `unbond` call

    // let resp = staking_builtin_client
    //     .chill()
    //     .send_recv(builtins_broker_pid)
    //     .await
    //     .expect("Failed to send chill request");

    // assert_eq!(resp, Ok(Vec::<u8>::new()));

    // todo [sab] invalid gas count for `withdraw_unbonded` call

    // let resp = staking_builtin_client
    //     .nominate(vec![H256::random().into()])
    //     .send_recv(builtins_broker_pid)
    //     .await
    //     .expect("Failed to send nominate request")
    //     .unwrap_err();

    // assert!(resp.to_string().contains("InsufficientBond"));

    // todo [sab] invalid gas count for `payout_stakers` call
}

// #[tokio::test]
// async fn call_bls381_builtin_from_app() {
//     let api = GearApi::dev().await.unwrap();

//     let (code_id, _) = api
//         .upload_code(WASM_BINARY)
//         .await
//         .expect("Failed to upload program code");

//     let remoting = GClientRemoting::new(api.clone());

//     let builtins_broker_pid = BuiltinsExampleClientFactory::new(remoting.clone())
//         .new()
//         .send_recv(code_id, b"builtins-example-app")
//         .await
//         .expect("Failed program init message");

//     api.transfer_keep_alive(builtins_broker_pid, 10_000 * ONE_VARA)
//         .await
//         .expect("Failed to transfer funds to program");

//     let mut bls381_builtin_client = Bls381Broker::new(remoting.clone());

//     let resp = bls381_builtin_client
//         .multi_miller_loop(vec![1, 2, 3].encode(), vec![4, 5, 6].encode())
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send multi_miller_loop request");

//     assert!(matches!(resp, Ok(Bls381Response::MultiMillerLoop(_))));

//     let resp = bls381_builtin_client
//         .final_exponentiation(vec![7, 8, 9].encode())
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send final_exponentiation request");

//     assert!(matches!(resp, Ok(Bls381Response::FinalExponentiation(_))));

//     let resp = bls381_builtin_client
//         .multi_scalar_multiplication_g_1(vec![10, 11].encode(), vec![12, 13].encode())
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send multi_scalar_multiplication_g1 request");

//     assert!(matches!(resp, Ok(Bls381Response::MultiScalarMultiplicationG1(_))));

//     let resp = bls381_builtin_client
//         .multi_scalar_multiplication_g_2(vec![14, 15].encode(), vec![16, 17].encode())
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send multi_scalar_multiplication_g2 request");

//     assert!(matches!(resp, Ok(Bls381Response::MultiScalarMultiplicationG2(_))));

//     let resp = bls381_builtin_client
//         .projective_multiplication_g_1(vec![18, 19].encode(), vec![20, 21].encode())
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send projective_multiplication_g1 request");

//     assert!(matches!(resp, Ok(Bls381Response::ProjectiveMultiplicationG1(_))));

//     let resp = bls381_builtin_client
//         .projective_multiplication_g_2(vec![22, 23].encode(), vec![24, 25].encode())
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send projective_multiplication_g2 request");

//     assert!(matches!(resp, Ok(Bls381Response::ProjectiveMultiplicationG2(_))));

//     let resp = bls381_builtin_client
//         .aggregate_g_1(vec![26, 27].encode())
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send aggregate_g_1 request");

//     assert!(matches!(resp, Ok(Bls381Response::AggregateG1(_))));

//     let resp = bls381_builtin_client
//         .map_to_g_2_affine(vec![28, 29].encode())
//         .send_recv(builtins_broker_pid)
//         .await
//         .expect("Failed to send map_to_g_2_affine request");

//     assert!(matches!(resp, Ok(Bls381Response::MapToG2Affine(_))));
// }
