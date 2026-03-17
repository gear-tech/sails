use partial_idl_client::*;
use sails_rs::{client::*, gtest::System, prelude::*};

const ACTOR_ID: u64 = 42;
#[cfg(debug_assertions)]
pub(crate) const APP_WASM_PATH: &str = "../../../target/wasm32-gear/debug/partial_idl_app.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const APP_WASM_PATH: &str =
    "../../../target/wasm32-gear/release/partial_idl_app.opt.wasm";

fn create_env() -> (GtestEnv, CodeId) {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
    let code_id = system.submit_code_file(APP_WASM_PATH);

    let env = GtestEnv::new(system, ACTOR_ID.into());
    (env, code_id)
}

#[tokio::test]
async fn test_partial_client_calls_second_method() {
    use partial_idl_client::partial_idl_service::PartialIdlService as _;
    let (env, code_id) = create_env();
    let program = env.deploy(code_id, vec![]).new().await.unwrap();
    let mut client = program.partial_idl_service();
    // Calling 'second' method which has @entry-id: 1 in our partial IDL
    let result: u32 = client.second(21).await.unwrap();

    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_partial_client_subscribes_to_third_event() {
    use partial_idl_client::partial_idl_service::{
        PartialIdlService as _, events::PartialIdlServiceEvents,
    };
    use sails_rs::futures::StreamExt as _;

    let (env, code_id) = create_env();
    let program = env.deploy(code_id, vec![]).new().await.unwrap();
    let mut client = program.partial_idl_service();
    let listener = client.listener();
    let mut events = listener.listen().await.unwrap();
    client.third().await.unwrap();
    let event = events.next().await.unwrap();
    assert_eq!(
        (
            program.id(),
            PartialIdlServiceEvents::ThirdDone("Third".to_string())
        ),
        event
    );
}
