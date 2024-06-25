use gclient::GearApi;
use ping_client::traits::{Ping, PingFactory, PingListener};
use sails_rtl::{
    calls::{Action, Activation, Call, Query},
    errors::RtlError,
    event_listener::{Listen, Subscribe},
    gsdk::calls::{GSdkArgs, GSdkRemoting},
};

mod ping_client;

const PING_PROGRAM_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/ping.opt.wasm";

#[tokio::test]
async fn ping_succeed() {
    let api = GearApi::dev().await.unwrap();
    let gas_limit = api.block_gas_limit().unwrap();
    let (code_id, ..) = api
        .upload_code_by_path(PING_PROGRAM_WASM_PATH)
        .await
        .unwrap();
    let remoting = GSdkRemoting::new(api);

    let mut factory = ping_client::PingFactory::new(&remoting);
    let activation = factory
        .new()
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .publish(code_id, "123")
        .await
        .unwrap();

    let program_id = activation.reply().await.unwrap();

    let mut ping_listener = ping_client::PingListener::new(&remoting).listener();
    let mut listener = ping_listener.subscribe(program_id).await.unwrap();

    let mut client = ping_client::Ping::new(&remoting);
    let call = client
        .ping("ping".to_owned())
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .publish(program_id)
        .await
        .unwrap();
    let reply = call.reply().await.unwrap();

    assert_eq!(Ok("pong".to_owned()), reply);

    //let ev = listener.wait_many(|ev| true, |_| true).await.unwrap();
    let ev = listener.next_event().await.unwrap();
    assert_eq!(ping_client::PingEvents::Ping, ev);

    let ev = listener.next_event().await.unwrap();
    assert_eq!(ping_client::PingEvents::Pong, ev);
}

#[tokio::test]
async fn ping_not_enough_gas() {
    let api = GearApi::dev_from_path(env!("GEAR_PATH")).await.unwrap();
    let (code_id, ..) = api
        .upload_code_by_path(PING_PROGRAM_WASM_PATH)
        .await
        .unwrap();
    let remoting = GSdkRemoting::new(api);

    let mut factory = ping_client::PingFactory::new(&remoting);
    let activation = factory.new().publish(code_id, "123").await.unwrap();

    let activation_reply = activation.reply().await;

    assert!(matches!(
        activation_reply,
        Err(sails_rtl::errors::Error::Rtl(
            RtlError::ReplyHasErrorString(s)
        )) if s.as_str() == "Not enough gas to handle program data"
    ));
}
