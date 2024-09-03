use demo_client::{counter::events::CounterEvents, dog::events::DogEvents, ping_pong, traits::*};
use fixture::Fixture;
use futures::stream::StreamExt;
use gstd::errors::{ErrorReplyReason, SimpleExecutionError};
use sails_rs::{
    calls::*,
    errors::RtlError,
    events::*,
    gtest::{
        calls::{BlockRunMode, GTestRemoting},
        System,
    },
};

mod fixture;

#[tokio::test]
async fn counter_add_works() {
    // Arrange

    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .new(Some(42), None)
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut counter_client = fixture.counter_client();
    // Listen to Counter events
    let mut counter_listener = fixture.counter_listener();
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Act

    // Use generated client code for calling Counter service
    // using the `send_recv` method
    let result = counter_client
        .add(10)
        .send_recv(demo_program_id)
        .await
        .unwrap();

    // Asert

    let event = counter_events.next().await.unwrap();

    assert_eq!(result, 52);
    assert_eq!((demo_program_id, CounterEvents::Added(10)), event);
}

#[tokio::test]
async fn counter_sub_works() {
    // Arrange

    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send`/`recv` pair
    // of methods
    let activation = demo_factory
        .new(Some(42), None)
        .send(fixture.demo_code_id(), "123")
        .await
        .unwrap();
    let demo_program_id = activation.recv().await.unwrap();

    let mut counter_client = fixture.counter_client();
    // Listen to Counter events
    let mut counter_listener = fixture.counter_listener();
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Act

    // Use generated client code for calling Counter service
    // using the `send`/`recv` pair of methods
    let response = counter_client.sub(10).send(demo_program_id).await.unwrap();
    let result = response.recv().await.unwrap();

    // Assert

    let event = counter_events.next().await.unwrap();

    assert_eq!(result, 32);
    assert_eq!((demo_program_id, CounterEvents::Subtracted(10)), event);
}

#[tokio::test]
async fn counter_query_works() {
    // Arrange
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .new(Some(42), None)
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let counter_client = fixture.counter_client();

    // Act

    // Use generated client code for query Counter service using the `recv` method
    let result = counter_client.value().recv(demo_program_id).await.unwrap();

    // Asert
    assert_eq!(result, 42);
}

#[tokio::test]
async fn counter_query_not_enough_gas() {
    // Arrange
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .new(Some(42), None)
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let counter_client = fixture.counter_client();

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

#[tokio::test]
async fn ping_pong_works() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    // Use generated client code for activating Demo program
    // using the `default` constructor and the `send_recv` method
    let demo_program_id = demo_factory
        .default()
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let demo_program = fixture.demo_program(demo_program_id);

    // Use generated `io` module for encoding/decoding calls and replies
    // and send/receive bytes using `gtest` native means
    let ping_call_payload = ping_pong::io::Ping::encode_call("ping".into());

    let message_id = demo_program.send_bytes(fixture.admin_id(), ping_call_payload);
    let run_result = fixture.run_next_block();

    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(message_id))
        .unwrap();

    let ping_reply_payload = reply_log_record.payload();

    let ping_reply = ping_pong::io::Ping::decode_reply(ping_reply_payload).unwrap();

    assert_eq!(ping_reply, Ok("pong".to_string()));
}

#[tokio::test]
async fn dog_barks() {
    // Arrange

    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(None, Some((1, -1)))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut dog_client = fixture.dog_client();
    let mut dog_listener = fixture.dog_listener();
    let mut dog_events = dog_listener.listen().await.unwrap();

    // Act

    let result = dog_client
        .make_sound()
        .send_recv(demo_program_id)
        .await
        .unwrap();

    // Assert

    let event = dog_events.next().await.unwrap();

    assert_eq!(result, "Woof! Woof!");
    assert_eq!((demo_program_id, DogEvents::Barked), event);
}

#[tokio::test]
async fn dog_walks() {
    // Arrange

    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(None, Some((1, -1)))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut dog_client = fixture.dog_client();
    let mut dog_listener = fixture.dog_listener();
    let mut dog_events = dog_listener.listen().await.unwrap();

    // Act

    dog_client
        .walk(10, 20)
        .send_recv(demo_program_id)
        .await
        .unwrap();

    // Assert

    let position = dog_client.position().recv(demo_program_id).await.unwrap();
    let event = dog_events.next().await.unwrap();

    assert_eq!(position, (11, 19));
    assert_eq!(
        (
            demo_program_id,
            DogEvents::Walked {
                from: (1, -1),
                to: (11, 19)
            }
        ),
        event
    );
}

#[tokio::test]
async fn dog_weights() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(None, Some((1, -1)))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let dog_client = fixture.dog_client();

    let avg_weight = dog_client.avg_weight().recv(demo_program_id).await.unwrap();

    assert_eq!(avg_weight, 42);
}

#[tokio::test]
async fn references_add() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(None, Some((1, -1)))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut client = fixture.references_client();

    let value = client.add(42).send_recv(demo_program_id).await.unwrap();

    assert_eq!(42, value);
}

#[tokio::test]
async fn references_bytes() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(None, Some((1, -1)))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut client = fixture.references_client();

    _ = client
        .add_byte(42)
        .send_recv(demo_program_id)
        .await
        .unwrap();
    _ = client
        .add_byte(89)
        .send_recv(demo_program_id)
        .await
        .unwrap();
    _ = client
        .add_byte(14)
        .send_recv(demo_program_id)
        .await
        .unwrap();

    let last = client.last_byte().recv(demo_program_id).await.unwrap();
    assert_eq!(Some(14), last);
}

#[tokio::test]
async fn references_guess_num() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(None, Some((1, -1)))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut client = fixture.references_client();

    let res1 = client
        .guess_num(42)
        .send_recv(demo_program_id)
        .await
        .unwrap();
    let res2 = client
        .guess_num(89)
        .send_recv(demo_program_id)
        .await
        .unwrap();
    let res3 = client.message().recv(demo_program_id).await.unwrap();
    let res4 = client.set_num(14).send_recv(demo_program_id).await.unwrap();
    let res5 = client
        .guess_num(14)
        .send_recv(demo_program_id)
        .await
        .unwrap();

    assert_eq!(Ok("demo".to_owned()), res1);
    assert_eq!(Err("Number is too large".to_owned()), res2);
    assert_eq!(Some("demo".to_owned()), res3);
    assert_eq!(Ok(()), res4);
    assert_eq!(Ok("demo".to_owned()), res5);
}

#[tokio::test]
async fn counter_add_manual_mode_works() {
    // Arrange
    const DEMO_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/demo.opt.wasm";
    let system = System::new();
    system.init_logger();
    system.mint_to(fixture::ADMIN_ID, 100_000_000_000_000);
    let demo_code_id = system.submit_code_file(DEMO_WASM_PATH);

    let remoting =
        GTestRemoting::new_from_system(system, fixture::ADMIN_ID.into(), BlockRunMode::Manual);

    let demo_factory = demo_client::DemoFactory::new(remoting.clone());

    // Use generated client code for activating Demo program
    let activation = demo_factory
        .new(Some(42), None)
        .send(demo_code_id, "123")
        .await
        .unwrap();

    // Run next Block
    remoting.run_next_block();

    let demo_program_id = activation.recv().await.unwrap();

    let mut counter_client_add = demo_client::Counter::new(remoting.clone());
    let mut counter_client_sub = demo_client::Counter::new(remoting.clone());
    // Listen to Counter events
    let mut counter_listener = demo_client::counter::events::listener(remoting.clone());
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Use generated client code for calling Counter service
    let call_add = counter_client_add
        .add(10)
        .send(demo_program_id)
        .await
        .unwrap();
    let call_sub = counter_client_sub
        .sub(20)
        .send(demo_program_id)
        .await
        .unwrap();

    // Run next Block
    remoting.run_next_block();

    // Got replies
    let result_add = call_add.recv().await.unwrap();
    assert_eq!(result_add, 52);
    let result_sub = call_sub.recv().await.unwrap();
    assert_eq!(result_sub, 32);

    // Got events
    assert_eq!(
        (demo_program_id, CounterEvents::Added(10)),
        counter_events.next().await.unwrap()
    );
    assert_eq!(
        (demo_program_id, CounterEvents::Subtracted(20)),
        counter_events.next().await.unwrap()
    );
}
