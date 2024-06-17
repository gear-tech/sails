use gstd::Encode;
use gtest::Program;
use sails_rtl::{
    calls::{Action, Call},
    gtest::calls::{GTestArgs, GTestRemoting},
    NonZeroU256, NonZeroU64, H256,
};
use this_that_svc_client::{traits::Service, DoThatParam, ManyVariants};

mod this_that_svc_client;

const PROGRAM_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/debug/this_that_svc.opt.wasm";
const ADMIN_ID: u64 = 10;

#[tokio::test]
async fn call_this_succeed() {
    let remoting = GTestRemoting::new();

    let remoting_clone = remoting.clone();
    let program = Program::from_file(remoting_clone.system(), PROGRAM_WASM_PATH);
    program.send_bytes(ADMIN_ID, "New".encode());

    let client = this_that_svc_client::Service::new(remoting);
    let reply = client
        .this()
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(program.id())
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    assert_eq!(42, reply);
}

#[tokio::test]
async fn call_do_that_succeed() {
    let remoting = GTestRemoting::new();
    let remoting_clone = remoting.clone();
    let program = Program::from_file(remoting_clone.system(), PROGRAM_WASM_PATH);
    program.send_bytes(ADMIN_ID, "New".encode());

    let param = DoThatParam {
        p1: 42,
        p2: "hello".to_owned(),
        p3: ManyVariants::Five((
            "world".to_owned(),
            H256::random(),
            NonZeroU64::MAX,
            NonZeroU256::MAX,
        )),
    };

    let mut client = this_that_svc_client::Service::new(remoting);
    let reply = client
        .do_that(param)
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(program.id())
        .await
        .unwrap()
        .reply()
        .await
        .unwrap()
        .unwrap();

    assert_eq!(("hello".to_owned(), 42), reply);
}
