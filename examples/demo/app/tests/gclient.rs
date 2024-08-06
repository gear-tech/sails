use demo_client::{counter::events::*, ping_pong, traits::*};
use futures::stream::StreamExt;
use gclient::GearApi;
use gstd::errors::{ErrorReplyReason, SimpleExecutionError};
use sails_rs::{calls::*, errors::RtlError, events::*, gsdk::calls::*, prelude::*};
use std::panic;

const DEMO_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/demo.opt.wasm";

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_add_works() {
    // Arrange

    let (remoting, demo_code_id, gas_limit) = spin_up_node_with_demo_code().await;

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .new(Some(42), None)
        .with_gas_limit(gas_limit)
        .send_recv(demo_code_id, "123")
        .await
        .unwrap();

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

    let event = counter_events.next().await.unwrap();

    assert_eq!(result, 52);
    assert_eq!((demo_program_id, CounterEvents::Added(10)), event);
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_sub_works() {
    // Arrange

    let (remoting, demo_code_id, gas_limit) = spin_up_node_with_demo_code().await;

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

    let (remoting, demo_code_id, gas_limit) = spin_up_node_with_demo_code().await;

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
    // and send/receive bytes using `gsdk` native means (remoting is just a wrapper)
    let ping_call_payload = ping_pong::io::Ping::encode_call("ping".into());

    // Act

    let ping_reply_payload = remoting
        .message(
            demo_program_id,
            ping_call_payload,
            Some(gas_limit),
            0,
            GSdkArgs,
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
        Err(sails_rs::errors::Error::Rtl(
            RtlError::ReplyHasErrorString(s)
        )) if s.as_str() == "Not enough gas to handle program data"
    ));
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn counter_query_works() {
    // Arrange

    let (remoting, demo_code_id, gas_limit) = spin_up_node_with_demo_code().await;

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
async fn counter_query_not_enough_gas() {
    // Arrange

    let (remoting, demo_code_id, gas_limit) = spin_up_node_with_demo_code().await;

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
            ErrorReplyReason::Execution(SimpleExecutionError::RanOutOfGas)
        )))
    ));
}

async fn spin_up_node_with_demo_code() -> (GSdkRemoting, CodeId, GasUnit) {
    let gear_path = option_env!("GEAR_PATH");
    if gear_path.is_none() {
        panic!("the 'GEAR_PATH' environment variable was not set during compile time");
    }
    let api = GearApi::dev_from_path(gear_path.unwrap()).await.unwrap();
    let gas_limit = api.block_gas_limit().unwrap();
    let remoting = GSdkRemoting::new(api);
    let code_id = remoting.upload_code_by_path(DEMO_WASM_PATH).await.unwrap();
    (remoting, code_id, gas_limit)
}
