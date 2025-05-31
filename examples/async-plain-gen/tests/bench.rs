use async_plain_gen_client::{
    PlainFactory, async_service::io::SomeAsyncMethod, no_async_service::io::SomeMethod,
    traits::PlainFactory as _,
};
use gtest::{System, constants::DEFAULT_USER_ALICE};
use sails_rs::calls::{ActionIo, Activation};
use sails_rs::prelude::Decode;
use sails_rs::{Encode, gtest::calls::GTestRemoting};

const WASM_PATH: &str = "../../target/wasm32-gear/debug/async_plain_gen.opt.wasm";

#[tokio::test]
async fn simple_bench() {
    let sys = System::new();
    sys.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    sys.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

    let code_id = sys.submit_code_file(WASM_PATH);

    // Initialize the program
    let remoting = GTestRemoting::new(sys, DEFAULT_USER_ALICE.into());
    let factory = PlainFactory::new(remoting.clone());
    let pid = factory
        .new()
        .send_recv(code_id, b"some_salt")
        .await
        .expect("Failed to create program");

    let remoting_sys = remoting.system();
    let program = remoting_sys
        .get_program(pid)
        .expect("Failed to get program");

    // Call async service
    let payload = SomeAsyncMethod::encode_call();

    let mid = program.send_bytes(DEFAULT_USER_ALICE, SomeAsyncMethod::encode_call());
    let block_res = remoting_sys.run_next_block();
    assert!(block_res.succeed.contains(&mid));

    let reply = block_res
        .log
        .iter()
        .find_map(|log| {
            if log.reply_to() == Some(mid) {
                Some(log.payload().to_vec())
            } else {
                None
            }
        })
        .expect("failed to find reply");
    let decoded: String =
        SomeAsyncMethod::decode_reply(&mut &reply[..]).expect("Failed to decode reply");

    println!("{decoded}");

    // assert_eq!(decoded, "This is an asynchronous method".to_string());

    let gas = block_res
        .gas_burned
        .get(&mid)
        .expect("Failed to get gas burned");
    println!("{gas}"); // GAS 1138233307

    // Call sync service
    let mid = program.send_bytes(DEFAULT_USER_ALICE, SomeMethod::encode_call());
    let block_res = remoting_sys.run_next_block();
    assert!(block_res.succeed.contains(&mid));

    let reply = block_res
        .log
        .iter()
        .find_map(|log| {
            if log.reply_to() == Some(mid) {
                Some(log.payload().to_vec())
            } else {
                None
            }
        })
        .expect("failed to find reply");
    let decoded: String = SomeMethod::decode_reply(&mut &reply).expect("Failed to decode reply");
    // assert_eq!(decoded, "This is a synchronous method".to_string());

    println!("{decoded}");

    let gas = block_res
        .gas_burned
        .get(&mid)
        .expect("Failed to get gas burned");
    println!("{gas}"); // GAS 959520092
}

/*
Fibo (23)
Async some method
10987237122
Non async some method
10808306252

Fibo(10)
Async some method
1158261265
Non async some method
979330395
*/
