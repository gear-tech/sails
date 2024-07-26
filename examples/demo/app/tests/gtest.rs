use demo_client::{
    counter::events::CounterEvents,
    dog::events::DogEvents,
    ping_pong,
    traits::{Counter, DemoFactory, Dog, References},
};
use fixture::Fixture;
use futures::stream::StreamExt;
use sails_rs::{calls::*, events::*};

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
async fn counter_value_by_ref_works() {
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
    let val = counter_client
        .value_by_ref()
        .recv(demo_program_id)
        .await
        .unwrap();

    // Assert
    assert_eq!(val, 42);
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

    let run_result = demo_program.send_bytes(fixture.admin_id(), ping_call_payload);

    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(run_result.sent_message_id()))
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

    let first = client.first_byte().recv(demo_program_id).await.unwrap();
    let last = client.last_byte().recv(demo_program_id).await.unwrap();
    assert_eq!(Some(42), first);
    assert_eq!(Some(14), last);
}
