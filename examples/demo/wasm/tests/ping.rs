use demo_client::traits::{DemoFactory, PingPong};
use fixture::Fixture;
use sails_rtl::{calls::*, gtest::calls::GTestArgs};

mod fixture;

#[tokio::test]
async fn ping_pong_works() {
    let fixture = Fixture::new(fixture::ADMIN_ID);

    let demo_factory = fixture.demo_factory();

    let demo_program_id = demo_factory
        .default()
        .with_args(GTestArgs::new(fixture.admin_id()))
        .execute(fixture.demo_code_id(), "123")
        .await
        .unwrap();

    let mut ping_pong_client = fixture.ping_pong_client();

    let result = ping_pong_client
        .ping("ping".into())
        .with_args(GTestArgs::new(fixture.admin_id()))
        .execute(demo_program_id)
        .await
        .unwrap();

    assert_eq!(result, Ok("pong".to_string()));
}
