mod ethapp_sol_client;

use ethapp_sol_client::{IEthappClient, Svc1DoThis};
use sails_rs::{
    ActorId,
    alloy_primitives::B256,
    alloy_sol_types::SolValue,
    client::{GstdEnv, GtestEnv},
    gear_core_errors::{ErrorReplyReason, ReplyCode, SimpleExecutionError},
    gtest::{Program, System},
};

#[cfg(debug_assertions)]
const WASM_PATH: &str = "../target/wasm32-gear/debug/ethapp.opt.wasm";
#[cfg(not(debug_assertions))]
const WASM_PATH: &str = "../target/wasm32-gear/release/ethapp.opt.wasm";

const ADMIN_ID: u64 = 10;

fn payload_client() -> IEthappClient<GstdEnv> {
    IEthappClient::new(GstdEnv, ActorId::zero())
}

fn assert_message_id(reply: [u8; 32]) {
    assert_ne!(reply, [0u8; 32]);
}

#[tokio::test]
async fn ethapp_sol_generated_client_payloads_work() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);

    let client = payload_client();
    let program = Program::from_file(&system, WASM_PATH);

    let payload = client.create_prg(false).encode_call();

    let message_id = program.send_bytes(ADMIN_ID, payload.as_slice());
    let run_result = system.run_next_block();
    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(message_id))
        .expect("no constructor reply found");
    assert!(matches!(
        reply_log_record.reply_code(),
        Some(ReplyCode::Success(_))
    ));

    let gas_burned = *run_result
        .gas_burned
        .get(&message_id)
        .expect("message not found");
    let wasm_size = std::fs::metadata(WASM_PATH).expect("read wasm metadata").len();
    println!("[ethapp_sol_generated_client_payloads_work] Init Gas: {gas_burned:>14}, Size: {wasm_size}");

    let payload = client.svc_1_do_this(false, 42u32, "hello".into()).encode_call();

    let message_id = program.send_bytes(ADMIN_ID, payload);
    let run_result = system.run_next_block();

    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(message_id))
        .expect("no method reply found");

    let reply = Svc1DoThis::decode_reply(0, reply_log_record.payload()).expect("typed decode reply");
    assert_message_id(reply);

    let gas_burned = *run_result
        .gas_burned
        .get(&message_id)
        .expect("message not found");
    println!("[ethapp_sol_generated_client_payloads_work] Handle Gas: {gas_burned:>14}, Size: {wasm_size}");
}

#[tokio::test]
async fn ethapp_remoting_with_generated_client_payloads_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    let code_id = system.submit_code_file(WASM_PATH);
    let env = GtestEnv::new(system, ADMIN_ID.into());

    let client = IEthappClient::new(env.clone(), ActorId::zero());

    let payload = client.create_prg(false).encode_call();
    let (program_id, _) = env
        .create_program(code_id, vec![], payload.as_slice(), Default::default())
        .expect("create program");

    let client = client.with_program_id(program_id);

    let reply = client
        .svc_1_do_this(false, 42u32, "hello".into())
        .send_for_reply()
        .expect("send for reply")
        .await
        .expect("reply");
    assert_message_id(reply);
}

#[tokio::test]
async fn ethapp_remoting_encode_reply_with_generated_client_payloads_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    let code_id = system.submit_code_file(WASM_PATH);
    let env = GtestEnv::new(system, ADMIN_ID.into());

    let client = IEthappClient::new(env.clone(), ActorId::zero());

    let payload = client.create_prg(true).encode_call();

    let (program_id, message_id) = env
        .create_program(code_id, vec![], payload.as_slice(), Default::default())
        .expect("create program");
    let reply_payload = env
        .message_reply_from_next_blocks(message_id)
        .await
        .expect("reply wait")
        .expect("reply value");

    let callback_selector = sails_rs::solidity::selector("replyOn_createPrg(bytes32)");
    assert_eq!(callback_selector.as_slice(), &reply_payload[..4]);
    let (_message_id,) = <(B256,)>::abi_decode_sequence(&reply_payload[4..]).expect("decode callback");

    let client = client.with_program_id(program_id);

    let payload = client.svc_1_do_this(true, 42u32, "hello".into()).encode_call();

    let reply_payload = env
        .send_for_reply(program_id, payload, Default::default())
        .await
        .expect("reply payload");

    let callback_selector = sails_rs::solidity::selector("replyOn_svc1DoThis(bytes32,uint32)");
    assert_eq!(callback_selector.as_slice(), &reply_payload[..4]);

    let (_message_id, result) =
        <(B256, u32)>::abi_decode_sequence(&reply_payload[4..]).expect("decode callback");
    assert_eq!(42, result);
}

#[tokio::test]
async fn ethapp_ctor_non_payable_fails_with_value_generated_client_payloads() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);

    let program = Program::from_file(&system, WASM_PATH);
    let client = payload_client();

    let payload = client.create_prg(false).encode_call();

    let message_id = program.send_bytes_with_value(ADMIN_ID, payload.as_slice(), 1000);
    let run_result = system.run_next_block();

    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(message_id))
        .expect("No reply found");

    if let Some(ReplyCode::Error(ErrorReplyReason::Execution(
        SimpleExecutionError::UserspacePanic,
    ))) = reply_log_record.reply_code()
    {
        let payload = reply_log_record.payload();
        let msg = String::from_utf8_lossy(payload);
        assert_eq!(msg, "panicked with ''create_prg' accepts no value'");
    } else {
        panic!(
            "Expected UserspacePanic, got {:?}",
            reply_log_record.reply_code()
        );
    }
}

#[tokio::test]
async fn ethapp_ctor_payable_works_with_value_generated_client_payloads() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);

    let program = Program::from_file(&system, WASM_PATH);
    let client = payload_client();

    let payload = client.create_payable(false).encode_call();

    let message_id = program.send_bytes_with_value(ADMIN_ID, payload.as_slice(), 1000);
    let run_result = system.run_next_block();

    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(message_id))
        .expect("No reply found");
    assert!(matches!(
        reply_log_record.reply_code(),
        Some(ReplyCode::Success(_))
    ));
}

#[tokio::test]
async fn ethapp_method_non_payable_fails_with_value_generated_client_payloads() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    let code_id = system.submit_code_file(WASM_PATH);
    let env = GtestEnv::new(system, ADMIN_ID.into());

    let client = IEthappClient::new(env.clone(), ActorId::zero());

    let payload = client.create_prg(false).encode_call();
    let (program_id, _) = env
        .create_program(code_id, vec![], payload.as_slice(), Default::default())
        .expect("create program");

    let client = client.with_program_id(program_id);

    let mut call = client
        .svc_1_do_this(false, 42u32, "hello".into())
        .with_params(|params| params.with_value(1000));
    let message_id = call.send_one_way().expect("send one way");

    let reply_log_record = env
        .message_reply_from_next_blocks(message_id)
        .await
        .expect("await reply channel")
        .expect_err("expected panic reply");

    match reply_log_record {
        sails_rs::client::GtestError::ReplyHasError(
            ErrorReplyReason::Execution(SimpleExecutionError::UserspacePanic),
            payload,
        ) => {
            let msg = String::from_utf8_lossy(&payload);
            assert_eq!(msg, "panicked with ''do_this' accepts no value'");
        }
        other => panic!("Expected UserspacePanic, got {other:?}"),
    }
}

#[tokio::test]
async fn ethapp_method_payable_works_with_value_generated_client_payloads() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    let code_id = system.submit_code_file(WASM_PATH);
    let env = GtestEnv::new(system, ADMIN_ID.into());

    let client = IEthappClient::new(env.clone(), ActorId::zero());

    let payload = client.create_prg(false).encode_call();
    let (program_id, _) = env
        .create_program(code_id, vec![], payload.as_slice(), Default::default())
        .expect("create program");

    let client = client.with_program_id(program_id);

    let reply = client
        .svc_1_do_this_payable(false, 42u32)
        .with_params(|params| params.with_value(1000))
        .send_for_reply()
        .expect("send for reply")
        .await
        .expect("reply");
    assert_message_id(reply);
}
