use gtest::{Log, Program, System};
use sails_rs::Encode;

const ACTOR_ID: u64 = 42;

#[tokio::test]
async fn event_routes_work() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ACTOR_ID, 1_000_000_000_000_000);

    let code_id = system.submit_code(event_routes_app::WASM_BINARY);
    let program_id = gtest::calculate_program_id(code_id, &[], None);
    let program = Program::from_binary_with_id(&system, program_id, event_routes_app::WASM_BINARY);
    _ = program.send_bytes(ACTOR_ID, ("New",).encode());

    _ = system.run_next_block();

    program.send_bytes(ACTOR_ID, ("Foo", "Foo").encode());
    program.send_bytes(ACTOR_ID, ("Bar", "Foo").encode());

    let run_result = system.run_next_block();

    let log_foo_start = Log::builder()
        .source(program_id)
        .dest(0)
        .payload_bytes(("Foo", "Start").encode());
    assert!(run_result.contains(&log_foo_start));

    let log_bar_start = Log::builder()
        .source(program_id)
        .dest(0)
        .payload_bytes(("Bar", "Start").encode());
    assert!(run_result.contains(&log_bar_start));

    let log = Log::builder().dest(ACTOR_ID).payload_bytes((2u8).encode());

    let _reply_id = system
        .get_mailbox(ACTOR_ID)
        .reply_bytes(log, &[], 0)
        .unwrap();

    let run_result = system.run_next_block();

    let log_foo_end = Log::builder()
        .source(program_id)
        .dest(0)
        .payload_bytes(("Foo", "End").encode());
    assert!(run_result.contains(&log_foo_end));

    let log = Log::builder().dest(ACTOR_ID).payload_bytes((2u8).encode());

    let _reply_id = system
        .get_mailbox(ACTOR_ID)
        .reply_bytes(log, &[], 0)
        .unwrap();

    let run_result = system.run_next_block();

    let log_bar_end = Log::builder()
        .source(program_id)
        .dest(0)
        .payload_bytes(("Bar", "End").encode());
    assert!(run_result.contains(&log_bar_end));
}
