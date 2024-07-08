use demo_client::{counter::events::*, traits::*};
use fixture::Fixture;
use futures::stream::StreamExt;
use sails_rtl::{calls::*, event_listener::*, gtest::calls::GTestArgs};
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
    let mut space = fixture.cloned_program_space();
    let mut remoting_listener = space.subscribe().await.unwrap();

    let factory = fixture.demo_factory();

    let program_id = factory
        .new(Some(42), None)
        .with_args(GTestArgs::new(fixture.admin_id()))
        .send_recv(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut counter_listener = fixture.counter_listener();
    // Typed service event listener
    let mut listener = counter_listener.subscribe(program_id).await.unwrap();

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

    let event = remoting_listener.next().await.unwrap();
    println!("{:?}", event);
    assert_eq!(CounterEvents::Added(2), decode_event(&event.1).unwrap());
    let event = remoting_listener.next().await.unwrap();
    println!("{:?}", event);
    assert_eq!(
        CounterEvents::Subtracted(1),
        decode_event(&event.1).unwrap()
    );

    let event = listener.next().await.unwrap();
    assert_eq!(CounterEvents::Added(2), event);
    let event = listener.next().await.unwrap();
    assert_eq!(CounterEvents::Subtracted(1), event);
}
