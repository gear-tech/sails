use demo_client::{counter::events::*, counter::*, value_fee::*, *};
use gclient::GearApi;
use gstd::errors::{ErrorReplyReason, SimpleExecutionError};
use sails_rs::{client::*, futures::StreamExt, prelude::*};
use std::panic;

#[cfg(debug_assertions)]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/debug/demo.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/release/demo.opt.wasm";

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_add_works() {
    // Arrange

    let (env, demo_code_id, gas_limit, gear_api) = spin_up_node_with_demo_code().await;
    let admin_id = ActorId::try_from(gear_api.account_id().encode().as_ref())
        .expect("failed to create actor id");

    // Use generated client code for activating Demo program
    // using the `new` constructor
    let demo_program = env
        .deploy::<DemoClientProgram>(demo_code_id, vec![])
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .await
        .unwrap();

    let initial_balance = gear_api.free_balance(admin_id).await.unwrap();

    let mut counter_client = demo_program.counter();
    // Listen to Counter events
    let counter_listener = counter_client.listener();
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Act

    // Use generated client code for calling Counter service
    let result = counter_client
        .add(10)
        .with_gas_limit(gas_limit)
        .await
        .unwrap();

    // Assert
    let balance = gear_api.free_balance(admin_id).await.unwrap();
    // initial_balance - balance = 287_416_465_000, release, node 1.8.0
    dbg!(initial_balance, balance, initial_balance - balance);

    let event = counter_events.next().await.unwrap();

    assert_eq!(result, 52);
    assert_eq!((demo_program.id(), CounterEvents::Added(10)), event);
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_sub_works() {
    // Arrange

    let (env, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send`/`recv` pair
    // of methods
    let demo_program = env
        .deploy::<DemoClientProgram>(demo_code_id, vec![])
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .await
        .unwrap();

    let mut counter_client = demo_program.counter();
    // Listen to Counter events
    let counter_listener = counter_client.listener();
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Act

    // Use generated client code for calling Counter service
    // using the `send`/`recv` pair of methods
    let result = counter_client
        .sub(10)
        .with_gas_limit(gas_limit)
        .await
        .unwrap();

    // Assert
    let event = counter_events.next().await.unwrap();

    assert_eq!(result, 32);
    assert_eq!((demo_program.id(), CounterEvents::Subtracted(10)), event);
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn ping_pong_works() {
    // Arrange

    let (env, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    // Use generated client code for activating Demo program
    // using the `default` constructor
    let demo_program = env
        .deploy::<DemoClientProgram>(demo_code_id, vec![])
        .default()
        .with_gas_limit(gas_limit)
        .await
        .unwrap();

    // Use generated `io` module for encoding/decoding calls and replies
    // and send/receive bytes using `gclient` native means (env is just a wrapper)
    let ping_call_payload =
        ping_pong::io::Ping::encode_params_with_prefix("PingPong", "ping".into());

    // Act
    let ping_reply_payload = env
        .send_for_reply(
            demo_program.id(),
            ping_call_payload,
            GclientParams::default().with_gas_limit(gas_limit),
        )
        .await
        .unwrap();

    let ping_reply =
        ping_pong::io::Ping::decode_reply_with_prefix("PingPong", ping_reply_payload).unwrap();

    // Assert

    assert_eq!(ping_reply, Ok("pong".to_string()));
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn demo_returns_not_enough_gas_on_activation() {
    // Arrange
    let (env, demo_code_id, ..) = spin_up_node_with_demo_code().await;

    // Act
    let demo_program = env
        .deploy::<DemoClientProgram>(demo_code_id, vec![])
        .default()
        .with_gas_limit(0)
        .await;

    // Assert
    assert!(matches!(
        demo_program,
        Err(GclientError::ReplyHasError(
            ErrorReplyReason::Execution(SimpleExecutionError::RanOutOfGas),
            _
        ))
    ));
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_query_works() {
    // Arrange

    let (env, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    // Use generated client code for activating Demo program
    // using the `new` constructor
    let demo_program = env
        .deploy::<DemoClientProgram>(demo_code_id, vec![])
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .await
        .unwrap();

    let counter_client = demo_program.counter();

    // Act

    // Use generated client code for query Counter service using the `query` method
    let result = counter_client.value().query().await.unwrap();

    // Assert
    assert_eq!(result, 42);
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_query_with_message_works() {
    // Arrange

    let (env, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    // Use generated client code for activating Demo program
    // using the `new` constructor
    let demo_program = env
        .deploy::<DemoClientProgram>(demo_code_id, vec![])
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .await
        .unwrap();

    let counter_client = demo_program.counter();

    // Act

    // Use generated client code for query Counter service
    let result = counter_client.value().await.unwrap();

    // Assert
    assert_eq!(result, 42);
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_query_not_enough_gas() {
    // Arrange

    let (env, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send_recv` method
    let demo_program = env
        .deploy::<DemoClientProgram>(demo_code_id, vec![])
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .await
        .unwrap();

    let counter_client = demo_program.counter();

    // Act

    // Use generated client code for query Counter service using the `recv` method
    let result = counter_client
        .value()
        .with_gas_limit(0) // Set gas_limit to 0
        .await;

    // Assert
    assert!(matches!(
        result,
        Err(GclientError::ReplyHasError(
            ErrorReplyReason::Execution(SimpleExecutionError::RanOutOfGas),
            _
        ))
    ));
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn value_fee_works() {
    // Arrange
    let (env, demo_code_id, _gas_limit, gear_api) = spin_up_node_with_demo_code().await;
    let admin_id = ActorId::try_from(gear_api.account_id().encode().as_ref())
        .expect("failed to create actor id");

    let demo_program = env
        .deploy::<DemoClientProgram>(demo_code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let initial_balance = gear_api.free_balance(admin_id).await.unwrap();
    let mut client = demo_program.value_fee();

    // Act

    // Use generated client code to call `do_something_and_take_fee` method with zero value
    let result = client.do_something_and_take_fee().await.unwrap();
    assert!(!result);

    // Use generated client code to call `do_something_and_take_fee` method with value
    let result = client
        .do_something_and_take_fee()
        .with_value(15_000_000_000_000)
        .await
        .unwrap();

    assert!(result);
    let fee = 10_000_000_000_000;
    let balance = gear_api.free_balance(admin_id).await.unwrap();
    dbg!(initial_balance, balance, initial_balance - balance - fee);
    // fee is 10_000_000_000_000 + spent gas
    // initial_balance - balance - fee = 546_866_717_300, release, node 1.8.0
    assert!(
        initial_balance - balance > 10_000_000_000_000
            && initial_balance - balance < 10_700_000_000_000
    );
}

async fn spin_up_node_with_demo_code() -> (GclientEnv, CodeId, GasUnit, GearApi) {
    let gear_path = option_env!("GEAR_PATH");
    if gear_path.is_none() {
        panic!("the 'GEAR_PATH' environment variable was not set during compile time");
    }
    let api = GearApi::dev_from_path(gear_path.unwrap()).await.unwrap();
    let gas_limit = api.block_gas_limit().unwrap();
    let (code_id, _) = api.upload_code_by_path(DEMO_WASM_PATH).await.unwrap();
    let remoting = GclientEnv::new(api.clone());
    (remoting, code_id, gas_limit, api)
}
