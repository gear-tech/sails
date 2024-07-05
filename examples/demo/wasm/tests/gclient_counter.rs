use demo_client::{counter::events::*, traits::*};
use gclient::GearApi;
use sails_rtl::{
    calls::*,
    errors::RtlError,
    event_listener::*,
    gsdk::calls::{GSdkArgs, GSdkRemoting},
};

const DEMO_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/demo.opt.wasm";

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn gclient_counter_works() {
    let gear_path = option_env!("GEAR_PATH");
    if gear_path.is_none() {
        return;
    }
    let api = GearApi::dev_from_path(gear_path.unwrap()).await.unwrap();
    let gas_limit = api.block_gas_limit().unwrap();
    let remoting = GSdkRemoting::new(api);
    let code_id = remoting.upload_code_by_path(DEMO_WASM_PATH).await.unwrap();

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
#[ignore = "requires run gear node on GEAR_PATH"]
async fn gclient_counter_not_enough_gas() {
    let gear_path = option_env!("GEAR_PATH");
    if gear_path.is_none() {
        return;
    }
    let api = GearApi::dev_from_path(gear_path.unwrap()).await.unwrap();
    let remoting = GSdkRemoting::new(api);
    let code_id = remoting.upload_code_by_path(DEMO_WASM_PATH).await.unwrap();

    let factory = demo_client::DemoFactory::new(remoting.clone());
    let activation_reply = factory.default().send_recv(code_id, "123").await;

    assert!(matches!(
        activation_reply,
        Err(sails_rtl::errors::Error::Rtl(
            RtlError::ReplyHasErrorString(s)
        )) if s.as_str() == "Not enough gas to handle program data"
    ));
}

#[tokio::test]
#[ignore = "requires run gear node on GEAR_PATH"]
async fn gclient_counter_events() {
    let gear_path = option_env!("GEAR_PATH");
    if gear_path.is_none() {
        return;
    }
    let api = GearApi::dev_from_path(gear_path.unwrap()).await.unwrap();
    let gas_limit = api.block_gas_limit().unwrap();
    let remoting = GSdkRemoting::new(api);
    let code_id = remoting.upload_code_by_path(DEMO_WASM_PATH).await.unwrap();

    // Low level remoting listener
    let mut remoting_cloned = remoting.clone();
    let mut remoting_listener = remoting_cloned.subscribe().await.unwrap();

    let factory = demo_client::DemoFactory::new(remoting.clone());
    let program_id = factory
        .new(Some(42), None)
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .send_recv(code_id, "123")
        .await
        .unwrap();

    let mut counter_listener = demo_client::counter::events::listener(remoting.clone());
    // Typed service event listener
    let mut listener = counter_listener.subscribe(program_id).await.unwrap();

    let mut client = demo_client::Counter::new(remoting.clone());
    let reply = client
        .add(2)
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .send_recv(program_id)
        .await
        .unwrap();

    assert_eq!(44, reply);

    let reply = client
        .value()
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(44, reply);

    let reply = client
        .sub(1)
        .with_args(GSdkArgs::default().with_gas_limit(gas_limit))
        .send_recv(program_id)
        .await
        .unwrap();

    assert_eq!(43, reply);

    let event = remoting_listener.next_event(|_| true).await.unwrap();
    println!("{:?}", event);
    assert_eq!(CounterEvents::Added(2), decode_event(&event.1).unwrap());
    let event = remoting_listener.next_event(|_| true).await.unwrap();
    println!("{:?}", event);
    assert_eq!(
        CounterEvents::Subtracted(1),
        decode_event(&event.1).unwrap()
    );

    let event = listener.next_event().await.unwrap();
    assert_eq!(CounterEvents::Added(2), event);
    let event = listener.next_event().await.unwrap();
    assert_eq!(CounterEvents::Subtracted(1), event);
}
