use gclient::GearApi;
/// These tests make interact with builtins from non-app (program) context via extrinsics (calls).
use sails_rs::{
    H256,
    builtins::{
        PROXY_BUILTIN_ID, ProxyBuiltin, ProxyType as SailsProxyType, RewardAccount,
        STAKING_BUILTIN_ID, StakingBuiltin,
    },
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

#[tokio::test]
async fn call_staking_builtin_with_extrinsic() {
    let api = GearApi::dev().await.unwrap();

    let remoting = GClientRemoting::new(api.clone());
    let staking_builtin_client = StakingBuiltin::new(remoting);

    let res = staking_builtin_client
        .bond(1_000_000_000_000, RewardAccount::None)
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send bond request");

    assert_eq!(res, Vec::<u8>::new());

    let res = staking_builtin_client
        .bond_extra(500_000_000_000)
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send bond_extra request");

    assert_eq!(res, Vec::<u8>::new());

    let res = staking_builtin_client
        .rebond(2_000_000_000_000)
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .unwrap_err()
        .to_string();
    assert!(res.contains("NoUnlockChunk"));

    let res = staking_builtin_client
        .set_payee(RewardAccount::Staked)
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send set_payee request");

    assert_eq!(res, Vec::<u8>::new());

    let balance = api.free_balance(api.account_id()).await;
    println!("Balance before staking: {balance:?}");

    let res = staking_builtin_client
        .unbond(200_000_000_000)
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send unbond request");

    assert_eq!(res, Vec::<u8>::new());

    let res = staking_builtin_client
        .chill()
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send chill request");

    assert_eq!(res, Vec::<u8>::new());

    let res = staking_builtin_client
        .withdraw_unbonded(1)
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send withdraw_unbonded request");

    assert_eq!(res, Vec::<u8>::new());

    let res = staking_builtin_client
        .nominate(vec![H256::random().into()])
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send nominate request");

    assert_eq!(res, Vec::<u8>::new());

    let res = staking_builtin_client
        .payout_stakers(H256::random().into(), 1)
        .send_recv(STAKING_BUILTIN_ID)
        .await
        .expect("Failed to send payout_stakers request");

    assert_eq!(res, Vec::<u8>::new());
}
