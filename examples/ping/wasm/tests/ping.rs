use gclient::GearApi;
use ping_client::traits::Ping;
use sails_rtl::{
    calls::{Activation, Call},
    gsdk::calls::{GSdkArgs, GSdkRemoting},
};

mod ping_client;

const PING_PROGRAM_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/ping.opt.wasm";

#[tokio::test]
async fn ping_succeed() {
    let api = GearApi::dev_from_path(env!("GEAR_PATH")).await.unwrap();
    let (code_id, ..) = api
        .upload_code_by_path(PING_PROGRAM_WASM_PATH)
        .await
        .unwrap();
    let remoting = GSdkRemoting::new(api);

    let activation = <ping_client::PingFactory as ping_client::traits::PingFactory<GSdkArgs>>::new(
        remoting.clone(),
    )
    .publish(code_id, "123")
    .await
    .unwrap();

    let program_id = activation.reply().await.unwrap();

    let mut client = ping_client::Ping::new(remoting);
    let call = client
        .ping("ping".to_owned())
        .publish(program_id)
        .await
        .unwrap();
    let reply = call.reply().await.unwrap();

    assert_eq!(Ok("pong".to_owned()), reply);
}
