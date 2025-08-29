use demo_client::env_client::{
    Demo as _, DemoCtors as _,
    counter::{Counter as _, events::CounterEvents},
};
use futures::StreamExt as _;
use sails_rs::{client::*, prelude::*};

const ACTOR_ID: u64 = 42;
#[cfg(debug_assertions)]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/debug/demo.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/release/demo.opt.wasm";

fn create_env() -> (GtestEnv, CodeId, GasUnit) {
    use sails_rs::gtest::{MAX_USER_GAS_LIMIT, System};

    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug,redirect=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
    // Submit program code into the system
    let code_id = system.submit_code_file(DEMO_WASM_PATH);

    // Create a remoting instance for the system
    // and set the block run mode to Next,
    // cause we don't receive any reply on `Exit` call
    let env = GtestEnv::new(system, ACTOR_ID.into()).with_block_run_mode(BlockRunMode::Next);
    (env, code_id, MAX_USER_GAS_LIMIT)
}

#[tokio::test]
async fn env_counter_add_works_via_next_mode() {
    let (env, code_id, _gas_limit) = create_env();

    // deploy DemoProgram
    let demo_program = env
        .deploy(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let mut counter_client = demo_program.counter();
    let mut counter_listener = counter_client.listener();
    let mut counter_events = counter_listener.listen().await.unwrap();

    assert_eq!(Ok(52), counter_client.add(10).await);
    assert_eq!(
        (demo_program.id(), CounterEvents::Added(10)),
        counter_events.next().await.unwrap()
    );
}
