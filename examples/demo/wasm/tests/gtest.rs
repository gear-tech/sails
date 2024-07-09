use demo_client::{counter::events::*, traits::*};
use fixture::Fixture;
use futures::stream::StreamExt;
use sails_rtl::{calls::*, events::*, gtest::calls::GTestArgs};

mod fixture;

#[tokio::test]
async fn counter_works() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(Some(42), None)
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut counter_client = fixture.counter_client();

    let result = counter_client
        .add(10)
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(demo_program_id)
        .await
        .unwrap();

    assert_eq!(result, 52);
}

#[tokio::test]
async fn counter_events() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    // Low level remoting listener
    let mut space = fixture.program_space().clone();
    let mut remoting_stream = space.listen().await.unwrap();

    let factory = fixture.demo_factory();

    let program_id = factory
        .new(Some(42), None)
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut counter_listener = fixture.counter_listener();
    // Typed service event listener
    let mut event_stream = counter_listener.listen().await.unwrap();

    let mut client = fixture.counter_client();
    let reply = client
        .add(2)
        .with_args(GTestArgs::default().with_actor_id(fixture.admin_id()))
        .send_recv(program_id)
        .await
        .unwrap();

    assert_eq!(44, reply);

    let reply = client
        .value()
        .with_args(GTestArgs::default().with_actor_id(fixture.admin_id()))
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(44, reply);

    let reply = client
        .sub(1)
        .with_args(GTestArgs::default().with_actor_id(fixture.admin_id()))
        .send_recv(program_id)
        .await
        .unwrap();

    assert_eq!(43, reply);

    let event = remoting_stream.next().await.unwrap();
    println!("{:?}", event);
    assert_eq!(
        (program_id, CounterEvents::Added(2)),
        (event.0, CounterEvents::decode_event(event.1).unwrap())
    );
    let event = remoting_stream.next().await.unwrap();
    println!("{:?}", event);
    assert_eq!(
        (program_id, CounterEvents::Subtracted(1)),
        (event.0, CounterEvents::decode_event(event.1).unwrap())
    );

    let event = event_stream.next().await.unwrap();
    assert_eq!((program_id, CounterEvents::Added(2)), event);
    let event = event_stream.next().await.unwrap();
    assert_eq!((program_id, CounterEvents::Subtracted(1)), event);
}

#[tokio::test]
async fn ping_pong_works() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .default()
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut ping_pong_client = fixture.ping_pong_client();

    let result = ping_pong_client
        .ping("ping".into())
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(demo_program_id)
        .await
        .unwrap();

    assert_eq!(result, Ok("pong".to_string()));
}

#[tokio::test]
async fn dog_barks() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(None, Some((1, -1)))
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut dog_client = fixture.dog_client();

    let result = dog_client
        .make_sound()
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(demo_program_id)
        .await
        .unwrap();

    assert_eq!(result, "Woof! Woof!");
}

#[tokio::test]
async fn dog_walks() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(None, Some((1, -1)))
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut dog_client = fixture.dog_client();

    dog_client
        .walk(10, 20)
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(demo_program_id)
        .await
        .unwrap();

    let position = dog_client
        .position()
        .with_args(GTestArgs::new(fixture.admin_id()))
        .recv(demo_program_id)
        .await
        .unwrap();

    assert_eq!(position, (11, 19));
    // TODO: Assert for Walked event as soon as event listener is implemented
}

#[tokio::test]
async fn dog_weights() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .new(None, Some((1, -1)))
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let dog_client = fixture.dog_client();

    let avg_weight = dog_client
        .avg_weight()
        .with_args(GTestArgs::new(fixture.admin_id()))
        .recv(demo_program_id)
        .await
        .unwrap();

    assert_eq!(avg_weight, 42);
}
