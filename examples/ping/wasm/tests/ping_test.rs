use gclient::GearApi;
use ping_client::traits::{Ping, PingFactory};
use sails_rtl::{
    calls::*,
    errors::RtlError,
    gsdk::calls::{GSdkArgs, GSdkRemoting},
};

mod ping_client;

const PING_PROGRAM_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/ping.opt.wasm";

#[tokio::test]
async fn ping_succeed() {
    let api = GearApi::dev_from_path(env!("GEAR_PATH")).await.unwrap();
    let gas_limit = api.block_gas_limit().unwrap();
    let (code_id, ..) = api
        .upload_code_by_path(PING_PROGRAM_WASM_PATH)
        .await
        .unwrap();
    let remoting = GSdkRemoting::new(api);

    let factory = ping_client::PingFactory::new(remoting.clone());
    let program_id = factory
        .new()
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .send_recv(code_id, "123")
        .await
        .unwrap();

    let mut client = ping_client::Ping::new(remoting.clone());
    let call = client
        .ping("ping".to_owned())
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .send_recv(program_id)
        .await
        .unwrap();

    assert_eq!(Ok("pong".to_owned()), call);
}

#[tokio::test]
async fn ping_not_enough_gas() {
    let api = GearApi::dev_from_path(env!("GEAR_PATH")).await.unwrap();
    let (code_id, ..) = api
        .upload_code_by_path(PING_PROGRAM_WASM_PATH)
        .await
        .unwrap();
    let remoting = GSdkRemoting::new(api);

    let factory = ping_client::PingFactory::new(remoting.clone());
    let activation_reply = factory.new().send_recv(code_id, "123").await;

    assert!(matches!(
        activation_reply,
        Err(sails_rtl::errors::Error::Rtl(
            RtlError::ReplyHasErrorString(s)
        )) if s.as_str() == "Not enough gas to handle program data"
    ));
}
