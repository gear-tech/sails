use ethapp_with_events::Events;
use sails_rs::{
    alloy_sol_types::SolValue,
    client::*,
    futures::StreamExt,
    gtest::{Program, System},
    prelude::*,
};

#[cfg(debug_assertions)]
pub(crate) const WASM_PATH: &str = "../target/wasm32-gear/debug/ethapp_with_events.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const WASM_PATH: &str = "../target/wasm32-gear/release/ethapp_with_events.opt.wasm";

pub(crate) const ADMIN_ID: u64 = 10;

#[tokio::test]
async fn ethapp_with_events_low_level_works() {
    // arrange
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);

    let program = Program::from_file(&system, WASM_PATH);

    let ctor = sails_rs::solidity::selector("create(bool)");
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
    println!("[ethapp_with_events_low_level_works] Init Gas: {gas_burned:>14}, Size: {wasm_size}");

    let do_this_sig = sails_rs::solidity::selector("svc1DoThis(bool,uint32,string)");
    let do_this_params = (false, 42, "hello").abi_encode_sequence();
    let payload = [do_this_sig.as_slice(), do_this_params.as_slice()].concat();

    // act
    let message_id = program.send_bytes(ADMIN_ID, payload);
    let run_result = system.run_next_block();

    // assert reply
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
    println!(
        "[ethapp_with_events_low_level_works] Handle Gas: {gas_burned:>14}, Size: {wasm_size}"
    );

    // assert event
    const ETH_EVENT_ADDR: ActorId = ActorId::new([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ]);
    let event_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.destination() == ETH_EVENT_ADDR)
        .unwrap();
    assert_eq!(program.id(), event_log_record.source());

    let event_payload = event_log_record.payload();
    assert_eq!(2u8, event_payload[0]);

    let sig = sails_rs::alloy_primitives::keccak256(b"DoThisEvent(uint32,string)");
    let topic1 = &event_payload[1..1 + 32];
    assert_eq!(sig.as_slice(), topic1);

    let hash2 = Events::topic_hash(&42u32);
    let topic2 = &event_payload[1 + 32..1 + 32 + 32];
    assert_eq!(hash2.as_slice(), topic2);

    let (s,): (String,) =
        SolValue::abi_decode_sequence(&event_payload[1 + 32 + 32..], false).unwrap();
    assert_eq!("hello", s);
}

#[tokio::test]
async fn ethapp_with_events_remoting_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    let code_id = system.submit_code_file(WASM_PATH);

    let env = GtestEnv::new(system, ADMIN_ID.into());
    let binding = env.clone();
    let mut listener = binding.listen(Some).await.unwrap();

    let ctor = sails_rs::solidity::selector("create(bool)");
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

    let (from, event_payload) = listener.next().await.unwrap();
    assert_eq!(from, program_id);
    assert_eq!(2u8, event_payload[0]);

    let sig = sails_rs::alloy_primitives::keccak256(b"DoThisEvent(uint32,string)");
    let topic1 = &event_payload[1..1 + 32];
    assert_eq!(sig.as_slice(), topic1);

    let hash2 = Events::topic_hash(&42u32);
    let topic2 = &event_payload[1 + 32..1 + 32 + 32];
    assert_eq!(hash2.as_slice(), topic2);

    let (s,): (String,) =
        SolValue::abi_decode_sequence(&event_payload[1 + 32 + 32..], false).unwrap();
    assert_eq!("hello", s);
}

#[tokio::test]
async fn ethapp_with_events_exposure_emit_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    let code_id = system.submit_code_file(WASM_PATH);

    let env = GtestEnv::new(system, ADMIN_ID.into());
    let binding = env.clone();
    let mut listener = binding.listen(Some).await.unwrap();

    let ctor = sails_rs::solidity::selector("create(bool)");
    let input = (false,).abi_encode_sequence();
    let payload = [ctor.as_slice(), input.as_slice()].concat();

    let (program_id, _) = env
        .create_program(code_id, vec![], payload.as_slice(), Default::default())
        .unwrap();

    let do_this_sig = sails_rs::solidity::selector("svc2DoThis(bool,uint32,string)");
    let do_this_params = (false, 42, "hello").abi_encode_sequence();
    let payload = [do_this_sig.as_slice(), do_this_params.as_slice()].concat();

    let reply_payload = env
        .send_for_reply(program_id, payload, Default::default())
        .await
        .unwrap();

    let reply = u32::abi_decode(reply_payload.as_slice(), true);
    assert_eq!(reply, Ok(42));

    // assert eth event
    let (from, event_payload) = listener.next().await.unwrap();
    assert_eq!(from, program_id);
    assert_eq!(2u8, event_payload[0]);

    let sig = sails_rs::alloy_primitives::keccak256(b"DoThisEvent(uint32,string)");
    let topic1 = &event_payload[1..1 + 32];
    assert_eq!(sig.as_slice(), topic1);

    let hash2 = Events::topic_hash(&42u32);
    let topic2 = &event_payload[1 + 32..1 + 32 + 32];
    assert_eq!(hash2.as_slice(), topic2);

    let (s,): (String,) =
        SolValue::abi_decode_sequence(&event_payload[1 + 32 + 32..], false).unwrap();
    assert_eq!("hello", s);

    // assert gear event
    let (from, event_payload) = listener.next().await.unwrap();
    assert_eq!(from, program_id);

    let (route, event_name, p1, p2): (String, String, u32, String) =
        Decode::decode(&mut event_payload.as_slice()).unwrap();
    assert_eq!(route, "Svc1");
    assert_eq!(event_name, "DoThisEvent");
    assert_eq!(p1, 42);
    assert_eq!(p2, "hello");
}
