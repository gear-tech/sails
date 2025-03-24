use ethapp_with_events::Events;
use sails_rs::{
    alloy_sol_types::SolValue,
    calls::Remoting,
    events::Listener,
    futures::StreamExt,
    gtest::{
        Program, System,
        calls::{GTestArgs, GTestRemoting},
    },
    prelude::*,
};

#[cfg(debug_assertions)]
pub(crate) const WASM_PATH: &str =
    "../target/wasm32-unknown-unknown/debug/ethapp_with_events.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const WASM_PATH: &str =
    "../target/wasm32-unknown-unknown/release/ethapp_with_events.opt.wasm";

pub(crate) const ADMIN_ID: u64 = 10;

#[tokio::test]
async fn ethapp_with_events_low_level_works() {
    // arrange
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 100_000_000_000_000);

    let program = Program::from_file(&system, WASM_PATH);

    let ctor = sails_rs::solidity::selector("default(uint128)");
    let input = (0u128,).abi_encode_params();
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

    let do_this_sig = sails_rs::solidity::selector("svc1_do_this(uint128,uint32,string)");
    let do_this_params = (0u128, 42, "hello").abi_encode_params();
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

    // assert event
    let dest = ActorId::new([0xff; 32]);
    let event_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.destination() == dest)
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
    system.mint_to(ADMIN_ID, 100_000_000_000_000);
    let code_id = system.submit_code_file(WASM_PATH);

    let remoting = GTestRemoting::new(system, ADMIN_ID.into());
    let mut binding = remoting.clone();
    let mut listener = binding.listen().await.unwrap();

    let ctor = sails_rs::solidity::selector("default(uint128)");
    let input = (0u128,).abi_encode_params();
    let payload = [ctor.as_slice(), input.as_slice()].concat();

    let (program_id, _) = remoting
        .clone()
        .activate(code_id, vec![], payload.as_slice(), 0, GTestArgs::default())
        .await
        .unwrap()
        .await
        .unwrap();

    let do_this_sig = sails_rs::solidity::selector("svc1_do_this(uint128,uint32,string)");
    let do_this_params = (0u128, 42, "hello").abi_encode_params();
    let payload = [do_this_sig.as_slice(), do_this_params.as_slice()].concat();

    let reply_payload = remoting
        .clone()
        .message(program_id, payload, 0, GTestArgs::default())
        .await
        .unwrap()
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
