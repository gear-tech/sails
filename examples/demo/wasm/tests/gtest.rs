use demo_client::traits::{Counter, DemoFactory, Dog, PingPong};
use fixture::Fixture;
use sails_rtl::{calls::*, gtest::calls::GTestArgs};

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
