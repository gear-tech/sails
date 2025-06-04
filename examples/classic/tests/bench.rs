use classic_client::{
    ClassicFactory, async_service::io::SomeAsyncMethod, no_async_service::io::SomeMethod,
    traits::ClassicFactory as _,
};
use gtest::{System, constants::DEFAULT_USER_ALICE};
use sails_rs::calls::{ActionIo, Activation};
use sails_rs::prelude::Decode;
use sails_rs::{Encode, gtest::calls::GTestRemoting};
const WASM_PATH: &str = "../../target/wasm32-gear/debug/classic.opt.wasm";

#[tokio::test]
async fn simple_bench() {
    let sys = System::new();
    sys.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    sys.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

    let code_id = sys.submit_code_file(WASM_PATH);

    // Initialize the program
    let remoting = GTestRemoting::new(sys, DEFAULT_USER_ALICE.into());
    let factory = ClassicFactory::new(remoting.clone());
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
    let mid = program.send_bytes(DEFAULT_USER_ALICE, payload);
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

    let gas = block_res
        .gas_burned
        .get(&mid)
        .expect("Failed to get gas burned");
    println!("{gas}"); // GAS 1218810257

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

    let decoded: String =
        SomeMethod::decode_reply(&mut &reply[..]).expect("Failed to decode reply");

    println!("{decoded}");

    let gas = block_res
        .gas_burned
        .get(&mid)
        .expect("Failed to get gas burned");
    println!("{gas}"); // GAS 1218577998
}

/*
Fibo(23)
Async some method
11068301698
Non async some method
11067684323

Fibo(10)
Async some method
1239325841
Non async some method
1238708466

try_handle sync in sails
Async some method
1262261914
Non async some method
1072831598
*/

/*
Selectable async in init

No async ctors
---- simple_bench stdout ----
Async some method
1181260435
Non async some method
998349648

Async ctors
Async some method
1337823334
Non async some method
1162937065
*/

/*
Non async init (!)
Async some method
1181374375
Non async some method
998463588

Async init (!)
Async some method
1193931031
Non async some method
1004600172

*/

// Generating handle_reply/signal gives more 70kk gas
// Async some method
// 1263562880
// Non async some method
// 1074197556
