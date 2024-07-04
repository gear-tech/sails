use demo_client::traits::{Counter, DemoFactory};
use gclient::GearApi;
use sails_rtl::{
    calls::*,
    errors::RtlError,
    gsdk::calls::{GSdkArgs, GSdkRemoting},
    CodeId,
};
use tokio::sync::OnceCell;

const DEMO_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/demo.opt.wasm";
static ONCE: OnceCell<CodeId> = OnceCell::const_new();

async fn demo_code_id(remoting: GSdkRemoting) -> &'static CodeId {
    ONCE.get_or_init(|| async { remoting.upload_code_by_path(DEMO_WASM_PATH).await })
        .await
}

#[tokio::test]
#[ignore = "requires running gear node"]
async fn gclient_counter_works() {
    let api = GearApi::dev().await.unwrap();
    let gas_limit = api.block_gas_limit().unwrap();
    let remoting = GSdkRemoting::new(api);
    let &code_id = demo_code_id(remoting.clone()).await;

    let factory = demo_client::DemoFactory::new(remoting.clone());
    let program_id = factory
        .new(Some(42), None)
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .send_recv(code_id, "123")
        .await
        .unwrap();

    let mut client = demo_client::Counter::new(remoting.clone());
    let result = client
        .add(10)
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .send_recv(program_id)
        .await
        .unwrap();

    assert_eq!(result, 52);
}

#[tokio::test]
#[ignore = "requires running gear node"]
async fn gclient_counter_not_enough_gas() {
    let api = GearApi::dev().await.unwrap();
    let remoting = GSdkRemoting::new(api);
    let &code_id = demo_code_id(remoting.clone()).await;

    let factory = demo_client::DemoFactory::new(remoting.clone());
    let activation_reply = factory.default().send_recv(code_id, "123").await;

    assert!(matches!(
        activation_reply,
        Err(sails_rtl::errors::Error::Rtl(
            RtlError::ReplyHasErrorString(s)
        )) if s.as_str() == "Not enough gas to handle program data"
    ));
}
