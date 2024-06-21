use counter_client::traits::{CounterFactory, Dec, Inc, Query};
use sails_rtl::{
    calls::{Action, Activation, Call},
    gtest::calls::{GTestArgs, GTestRemoting},
};

mod counter_client;

const PROGRAM_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/counter.opt.wasm";
const ADMIN_ID: u64 = 10;

#[tokio::test]
async fn counter_succeed() {
    let remoting = GTestRemoting::new();

    let code_id = remoting.system().submit_code_file(PROGRAM_WASM_PATH);

    let program_id = counter_client::CounterFactory::new(&remoting)
        .with_initial_value(10)
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(code_id, vec![])
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    let mut client = counter_client::Inc::new(remoting.clone());
    let reply = client
        .op(32)
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(program_id)
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    assert_eq!(10, reply);

    let client = counter_client::Query::new(remoting.clone());
    let reply = client
        .current_value()
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(program_id)
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    assert_eq!(42, reply);

    let mut client = counter_client::Dec::new(remoting.clone());
    let reply = client
        .op(1)
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(program_id)
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    assert_eq!(42, reply);
}
