use demo_client::{counter::events::*, ping_pong, traits::*};
use gclient::GearApi;
use gstd::errors::{ErrorReplyReason, SimpleExecutionError};
use sails_rs::{
    calls::*, errors::RtlError, events::*, futures::StreamExt, gclient::calls::*, prelude::*,
};
use std::panic;

#[cfg(debug_assertions)]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/debug/demo.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/release/demo.opt.wasm";

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_add_works() {
    // Arrange

    let (remoting, demo_code_id, gas_limit, gear_api) = spin_up_node_with_demo_code().await;
    let admin_id = ActorId::try_from(gear_api.account_id().encode().as_ref())
        .expect("failed to create actor id");

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .send_recv(demo_code_id, "123")
        .await
        .unwrap();

    let initial_balance = gear_api.free_balance(admin_id).await.unwrap();

    let mut counter_client = demo_client::Counter::new(remoting.clone());
    // Listen to Counter events
    let mut counter_listener = demo_client::counter::events::listener(remoting.clone());
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Act

    // Use generated client code for calling Counter service
    // using the `send_recv` method
    let result = counter_client
        .add(10)
        .with_gas_limit(gas_limit)
        .send_recv(demo_program_id)
        .await
        .unwrap();

    // Asert
    let balance = gear_api.free_balance(admin_id).await.unwrap();
    // initial_balance - balance = 287_416_465_000, release, node 1.8.0
    dbg!(initial_balance, balance, initial_balance - balance);

    let event = counter_events.next().await.unwrap();

    assert_eq!(result, 52);
    assert_eq!((demo_program_id, CounterEvents::Added(10)), event);
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_sub_works() {
    // Arrange

    let (remoting, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send`/`recv` pair
    // of methods
    let activation = demo_factory
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .send(demo_code_id, "123")
        .await
        .unwrap();
    let demo_program_id = activation.recv().await.unwrap();

    let mut counter_client = demo_client::Counter::new(remoting.clone());
    // Listen to Counter events
    let mut counter_listener = demo_client::counter::events::listener(remoting.clone());
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Act

    // Use generated client code for calling Counter service
    // using the `send`/`recv` pair of methods
    let response = counter_client
        .sub(10)
        .with_gas_limit(gas_limit)
        .send(demo_program_id)
        .await
        .unwrap();
    let result = response.recv().await.unwrap();

    // Assert

    let event = counter_events.next().await.unwrap();

    assert_eq!(result, 32);
    assert_eq!((demo_program_id, CounterEvents::Subtracted(10)), event);
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn ping_pong_works() {
    // Arrange

    let (remoting, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());

    // Use generated client code for activating Demo program
    // using the `default` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .default()
        .with_gas_limit(gas_limit)
        .send_recv(demo_code_id, "123")
        .await
        .unwrap();

    // Use generated `io` module for encoding/decoding calls and replies
    // and send/receive bytes using `gclient` native means (remoting is just a wrapper)
    let ping_call_payload = ping_pong::io::Ping::encode_call("ping".into());

    // Act

    let ping_reply_payload = remoting
        .message(
            demo_program_id,
            ping_call_payload,
            Some(gas_limit),
            0,
            GClientArgs::default(),
        )
        .await
        .unwrap()
        .await
        .unwrap();

    let ping_reply = ping_pong::io::Ping::decode_reply(ping_reply_payload).unwrap();

    // Assert

    assert_eq!(ping_reply, Ok("pong".to_string()));
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn demo_returns_not_enough_gas_on_activation() {
    // Arrange

    let (remoting, demo_code_id, ..) = spin_up_node_with_demo_code().await;

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());

    // Act

    let activation_result = demo_factory
        .default()
        .with_gas_limit(0)
        .send_recv(demo_code_id, "123")
        .await;

    // Assert

    assert!(matches!(
        activation_result,
        Err(sails_rs::errors::Error::Rtl(RtlError::ReplyHasErrorString(
            _message
        )))
    ));
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_query_works() {
    // Arrange

    let (remoting, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .send_recv(demo_code_id, "123")
        .await
        .unwrap();

    let counter_client = demo_client::Counter::new(remoting.clone());

    // Act

    // Use generated client code for query Counter service using the `recv` method
    let result = counter_client.value().recv(demo_program_id).await.unwrap();

    // Asert
    assert_eq!(result, 42);
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_query_with_message_works() {
    // Arrange

    let (remoting, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .send_recv(demo_code_id, "123")
        .await
        .unwrap();

    let counter_client = demo_client::Counter::new(remoting.clone());

    // Act

    // Use generated client code for query Counter service using the `recv` method
    // Set `query_with_message` to `true`
    let result = counter_client
        .value()
        .query_with_message(true)
        .recv(demo_program_id)
        .await
        .unwrap();

    // Asert
    assert_eq!(result, 42);
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_query_not_enough_gas() {
    // Arrange

    let (remoting, demo_code_id, gas_limit, ..) = spin_up_node_with_demo_code().await;

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .send_recv(demo_code_id, "123")
        .await
        .unwrap();

    let counter_client = demo_client::Counter::new(remoting.clone());

    // Act

    // Use generated client code for query Counter service using the `recv` method
    let result = counter_client
        .value()
        .with_gas_limit(0) // Set gas_limit to 0
        .recv(demo_program_id)
        .await;

    // Asert
    assert!(matches!(
        result,
        Err(sails_rs::errors::Error::Rtl(RtlError::ReplyHasError(
            ErrorReplyReason::Execution(SimpleExecutionError::RanOutOfGas),
            _payload
        )))
    ));
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn value_fee_works() {
    // Arrange
    let (remoting, demo_code_id, _gas_limit, gear_api) = spin_up_node_with_demo_code().await;
    let admin_id = ActorId::try_from(gear_api.account_id().encode().as_ref())
        .expect("failed to create actor id");

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());
    let program_id = demo_factory
        .new(Some(42), None)
        .send_recv(demo_code_id, "123")
        .await
        .unwrap();

    let initial_balance = gear_api.free_balance(admin_id).await.unwrap();
    let mut client = demo_client::ValueFee::new(remoting.clone());

    // Act

    // Use generated client code to call `do_something_and_take_fee` method with zero value
    let result = client
        .do_something_and_take_fee()
        .send_recv(program_id)
        .await
        .unwrap();
    assert!(!result);

    // Use generated client code to call `do_something_and_take_fee` method with value
    let result = client
        .do_something_and_take_fee()
        .with_value(15_000_000_000_000)
        .send_recv(program_id)
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

async fn spin_up_node_with_demo_code() -> (GClientRemoting, CodeId, GasUnit, GearApi) {
    let gear_path = option_env!("GEAR_PATH");
    if gear_path.is_none() {
        panic!("the 'GEAR_PATH' environment variable was not set during compile time");
    }
    let api = GearApi::dev_from_path(gear_path.unwrap()).await.unwrap();
    let gas_limit = api.block_gas_limit().unwrap();
    let (code_id, _) = api.upload_code_by_path(DEMO_WASM_PATH).await.unwrap();
    let remoting = GClientRemoting::new(api.clone());
    (remoting, code_id, gas_limit, api)
}
