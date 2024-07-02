use demo_client::traits::{Counter, DemoFactory};
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
        .publish(fixture.demo_code_id(), "123")
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    let mut counter_client = fixture.counter_client();

    let reply_ticket = counter_client
        .add(10)
        .with_args(GTestArgs::new(fixture.admin_id()))
        .publish(demo_program_id)
        .await
        .unwrap();

    assert_eq!(reply_ticket.reply().await.unwrap(), 52);
}
