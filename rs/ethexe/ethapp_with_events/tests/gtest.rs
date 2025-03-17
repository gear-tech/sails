use sails_rs::{
    alloy_sol_types::SolValue,
    gtest::{Program, System},
};
#[cfg(debug_assertions)]
pub(crate) const WASM_PATH: &str =
    "../target/wasm32-unknown-unknown/debug/ethapp_with_events.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const WASM_PATH: &str =
    "../target/wasm32-unknown-unknown/release/ethapp_with_events.opt.wasm";

pub(crate) const ADMIN_ID: u64 = 10;

#[tokio::test]
async fn ethapp_sol_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 100_000_000_000_000);

    let program = Program::from_file(&system, WASM_PATH);

    let ctor = sails_rs::solidity::selector("default(uint128)");
    let message_id = program.send_bytes(ADMIN_ID, ctor.as_slice());
    let run_result = system.run_next_block();
    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(message_id))
        .unwrap();
    assert!(matches!(
        reply_log_record.reply_code(),
        Some(sails_rs::gear_core_errors::ReplyCode::Success(_))
    ));

    let do_this_sig = sails_rs::solidity::selector("svc1_do_this(uint32,string,uint128)");
    let do_this_params = (42, "hello").abi_encode_params();
    let payload = [do_this_sig.as_slice(), do_this_params.as_slice()].concat();

    let message_id = program.send_bytes(ADMIN_ID, payload);
    let run_result = system.run_next_block();

    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(message_id))
        .unwrap();

    let reply_payload = reply_log_record.payload();
    let reply = u32::abi_decode(reply_payload, true);

    assert_eq!(reply, Ok(42));
}
