use gclient::{
    errors::{self, ModuleError},
    EventListener, EventProcessor, GearApi,
};
use ping_client::{traits::Ping, traits::PingFactory};
use sails_rtl::{
    calls::{Action, Activation, Call, Remoting},
    collections::BTreeMap,
    errors::Result,
    gsdk::calls::{GSdkArgs, GSdkRemoting},
    ActorId, CodeId, Decode, Encode,
};

mod ping_client;

const PING_PROGRAM_WASM_PATH: &str = "E:\\git\\sails\\examples\\ping\\wasm\\tests\\ping.opt.wasm";

#[tokio::test]
async fn ping_succeed() {
    let api = GearApi::dev().await.unwrap();
    let (code_id, ..) = api
        .upload_code_by_path(PING_PROGRAM_WASM_PATH)
        .await
        .unwrap();
    let code_id = CodeId::from(code_id.as_ref());

    let remoting = GSdkRemoting::dev().await.unwrap();

    let activation = <ping_client::PingFactory as ping_client::traits::PingFactory<GSdkArgs>>::new(
        remoting.clone(),
    )
    .with_args(GSdkArgs::default().with_gas_limit(Some(100_000_000_000)))
    .publish(code_id, "123")
    .await
    .unwrap();

    let program_id = activation.reply().await.unwrap();

    let mut client = ping_client::Ping::new(remoting);
    let call = client
        .ping("ping".to_owned())
        .with_args(GSdkArgs::default().with_gas_limit(Some(100_000_000_000)))
        .publish(program_id)
        .await
        .unwrap();
    let reply = call.reply().await.unwrap().unwrap();

    assert_eq!("pong", reply);
}
