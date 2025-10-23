use sails_rs::{
    alloy_primitives::B256,
    alloy_sol_types::SolValue,
    client::*,
    gtest::{Program, System},
};

#[cfg(debug_assertions)]
pub(crate) const WASM_PATH: &str = "../target/wasm32-gear/debug/ethapp.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const WASM_PATH: &str = "../target/wasm32-gear/release/ethapp.opt.wasm";

pub(crate) const ADMIN_ID: u64 = 10;

#[tokio::test]
async fn ethapp_sol_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);

    let program = Program::from_file(&system, WASM_PATH);

    let ctor = sails_rs::solidity::selector("createPrg(bool)");
    let input = (false,).abi_encode_sequence();
    let payload = [ctor.as_slice(), input.as_slice()].concat();

    let message_id = program.send_bytes(ADMIN_ID, payload.as_slice());
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

    let gas_burned = *run_result
        .gas_burned
        .get(&message_id)
        .expect("message not found");
    let wasm_size = std::fs::metadata(WASM_PATH).unwrap().len();
    println!("[ethapp_sol_works] Init Gas: {gas_burned:>14}, Size: {wasm_size}");

    let do_this_sig = sails_rs::solidity::selector("svc1DoThis(bool,uint32,string)");
    let do_this_params = (false, 42, "hello").abi_encode_sequence();
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

    let gas_burned = *run_result
        .gas_burned
        .get(&message_id)
        .expect("message not found");
    println!("[ethapp_sol_works] Handle Gas: {gas_burned:>14}, Size: {wasm_size}");
}

#[tokio::test]
async fn ethapp_remoting_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    let code_id = system.submit_code_file(WASM_PATH);
    let env = GtestEnv::new(system, ADMIN_ID.into());

    let ctor = sails_rs::solidity::selector("createPrg(bool)");
    let input = (false,).abi_encode_sequence();
    let payload = [ctor.as_slice(), input.as_slice()].concat();

    let (program_id, _) = env
        .create_program(code_id, vec![], payload.as_slice(), Default::default())
        .unwrap();

    let do_this_sig = sails_rs::solidity::selector("svc1DoThis(bool,uint32,string)");
    let do_this_params = (false, 42, "hello").abi_encode_sequence();
    let payload = [do_this_sig.as_slice(), do_this_params.as_slice()].concat();

    let reply_payload = env
        .send_for_reply(program_id, payload, Default::default())
        .await
        .unwrap();

    let reply = u32::abi_decode(reply_payload.as_slice(), true);
    assert_eq!(reply, Ok(42));
}

#[tokio::test]
async fn ethapp_remoting_encode_reply_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    let code_id = system.submit_code_file(WASM_PATH);
    let env = GtestEnv::new(system, ADMIN_ID.into());

    let ctor = sails_rs::solidity::selector("createPrg(bool)");
    let input = (true,).abi_encode_sequence();
    let payload = [ctor.as_slice(), input.as_slice()].concat();

    // act
    let (program_id, message_id) = env
        .create_program(code_id, vec![], payload.as_slice(), Default::default())
        .unwrap();
    let reply_payload = env
        .message_reply_from_next_blocks(message_id)
        .await
        .unwrap()
        .unwrap();

    // assert
    let callback_selector = sails_rs::solidity::selector("replyOn_createPrg(bytes32)");
    assert_eq!(callback_selector.as_slice(), &reply_payload[..4]);
    let (_message_id,) = <(B256,)>::abi_decode_sequence(&reply_payload[4..], false).unwrap();

    // arrange
    let do_this_sig = sails_rs::solidity::selector("svc1DoThis(bool,uint32,string)");
    let do_this_params = (true, 42, "hello").abi_encode_sequence();
    let payload = [do_this_sig.as_slice(), do_this_params.as_slice()].concat();

    // act
    let reply_payload = env
        .send_for_reply(program_id, payload, Default::default())
        .await
        .unwrap();

    // assert
    let callback_selector = sails_rs::solidity::selector("replyOn_svc1DoThis(bytes32,uint32)");
    assert_eq!(callback_selector.as_slice(), &reply_payload[..4]);

    let (_message_id, result) =
        <(B256, u32)>::abi_decode_sequence(&reply_payload[4..], false).unwrap();
    assert_eq!(42, result);
}
