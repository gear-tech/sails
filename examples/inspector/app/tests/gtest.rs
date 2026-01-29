use demo_client::{DemoClientCtors, DemoClientProgram};
use inspector_client::{
    InspectorClient, InspectorClientCtors, InspectorClientProgram,
    inspector::{Inspector as _, ValidationError},
};
use sails_rs::client::*;

const ACTOR_ID: u64 = 42;

#[cfg(debug_assertions)]
const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/debug/demo.opt.wasm";
#[cfg(not(debug_assertions))]
const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/release/demo.opt.wasm";

#[tokio::test]
async fn test_cross_contract_panic_handling() {
    let system = sails_rs::gtest::System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let demo_code = std::fs::read(DEMO_WASM_PATH).unwrap();
    let demo_code_id = system.submit_code(demo_code);
    let inspector_code_id = system.submit_code(inspector_app::WASM_BINARY);

    let env = GtestEnv::new(system, ACTOR_ID.into());

    let demo_program = env.deploy::<DemoClientProgram>(demo_code_id, vec![1]);
    let demo_program = demo_program.default().await.unwrap();

    let inspector_factory = env.deploy::<InspectorClientProgram>(inspector_code_id, vec![2]);
    let inspector_program = inspector_factory.new(demo_program.id()).await.unwrap();

    let mut inspector = inspector_program.inspector();

    let res = inspector.test_range_panic().await.unwrap();
    assert_eq!(res, Err(ValidationError::TooBig));

    let total = inspector.test_total_errors().await.unwrap();
    assert_eq!(total, 0, "State should rollback in Demo after panic");

    let res = inspector.test_nonzero_panic().await.unwrap();
    assert_eq!(res, Err("Value is zero".to_string()));

    let res = inspector.test_even_panic().await.unwrap();
    assert_eq!(res, Err(()));

    let total = inspector.test_total_errors().await.unwrap();
    assert_eq!(total, 0);
}
