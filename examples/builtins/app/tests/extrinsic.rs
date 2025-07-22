use gclient::GearApi;
/// These tests make interact with builtins from non-app (program) context via extrinsics (calls).
use sails_rs::{
    builtins::{
        Bls381Builtin, Bls381Request, ProxyBuiltin, ProxyType as SailsProxyType, RewardAccount, StakingBuiltin, BLS381_BUILTIN_ID, PROXY_BUILTIN_ID, STAKING_BUILTIN_ID
    }, calls::Call, gclient::calls::GClientRemoting, H256
};

const ONE_VARA: u128 = 1_000_000_000_000;

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

#[tokio::test]
async fn call_staking_builtin_with_extrinsic() {
    let api = GearApi::dev().await.unwrap();

    let remoting = GClientRemoting::new(api.clone());
    let staking_builtin_client = StakingBuiltin::new(remoting);

    let res = staking_builtin_client
        .bond(5 * ONE_VARA, RewardAccount::None)
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send bond request");

    assert_eq!(res, Vec::<u8>::new());

    // let res = staking_builtin_client
    //     .bond_extra(500_000_000_000)
    //     .send_recv(STAKING_BUILTIN_ID)
    //     .await
    //     .expect("Failed to send bond_extra request");

    // assert_eq!(res, Vec::<u8>::new());

    // let res = staking_builtin_client
    //     .rebond(2_000_000_000_000)
    //     .send_recv(STAKING_BUILTIN_ID)
    //     .await
    //     .unwrap_err()
    //     .to_string();
    // assert!(res.contains("NoUnlockChunk"));

    // let res = staking_builtin_client
    //     .set_payee(RewardAccount::Staked)
    //     .send_recv(STAKING_BUILTIN_ID)
    //     .await
    //     .expect("Failed to send set_payee request");

    // assert_eq!(res, Vec::<u8>::new());

    // todo [sab] invalid gas count for `unbond` call

    let _res = staking_builtin_client
        .unbond(2 * ONE_VARA)
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send unbond request");

    // let res = staking_builtin_client
    //     .chill()
    //     .send_recv(STAKING_BUILTIN_ID)
    //     .await
    //     .expect("Failed to send chill request");

    // assert_eq!(res, Vec::<u8>::new());

    // todo [sab] invalid gas count for `withdraw_unbonded` call

    // let res = staking_builtin_client
    //     .nominate(vec![H256::random().into()])
    //     .send_recv(STAKING_BUILTIN_ID)
    //     .await
    //     .unwrap_err()
    //     .to_string();

    // assert!(res.contains("InsufficientBond"));

    // let res = staking_builtin_client
    //     .payout_stakers(H256::random().into(), 1)
    //     .send_recv(STAKING_BUILTIN_ID)
    //     .await
    //     .unwrap_err()
    //     .to_string();

    // assert!(res.contains("NotStash"));
}

// #[tokio::test]
// async fn call_bls381_builtin_with_extrinsic() {
//     use sails_rs::Encode;
//     let api = GearApi::dev().await.unwrap();

//     let remoting = GClientRemoting::new(api.clone());
//     let bls381_builtin_client = Bls381Builtin::new(remoting);

//     let res = bls381_builtin_client
//         .multi_miller_loop(vec![1u8, 2, 3].encode(), vec![4u8, 5, 6].encode())
//         .send_recv(BLS381_BUILTIN_ID)
//         .await
//         .unwrap_err()
//         .to_string();

//     println!("Response: {:?}", res);
// }