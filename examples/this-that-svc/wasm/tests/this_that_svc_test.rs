use gclient::GearApi;
use sails_rtl::{
    calls::*,
    gsdk::calls::{GSdkArgs, GSdkRemoting},
};
use this_that_svc_client::traits::ThisThatSvc;

mod this_that_svc_client;

const PROGRAM_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/debug/this_that_svc.opt.wasm";

#[tokio::test]
async fn ping_succeed() {
    let api = GearApi::dev_from_path(env!("GEAR_PATH")).await.unwrap();
    let gas_limit = api.block_gas_limit().unwrap();
    let (code_id, ..) = api.upload_code_by_path(PROGRAM_WASM_PATH).await.unwrap();

    // Create program w/o constructor
    let (_, program_id, ..) = api
        .create_program_bytes(code_id, "123", vec![], gas_limit, 0)
        .await
        .unwrap();

    let remoting = GSdkRemoting::new(api);

    let client = this_that_svc_client::ThisThatSvc::new(remoting);
    let reply = client
        .this()
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(42, reply);
}
