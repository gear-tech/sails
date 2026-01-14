use gtest::{Log, Program, System};
use sails_rs::{Encode, header::SailsMessageHeader, meta::InterfaceId};

const ACTOR_ID: u64 = 42;

/// Test that event routes work as expected in async mode
#[tokio::test]
async fn event_routes_work() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ACTOR_ID, 1_000_000_000_000_000);

    let code_id = system.submit_code(event_routes_app::WASM_BINARY);
    let program_id = gtest::calculate_program_id(code_id, &[], None);
    let program = Program::from_binary_with_id(&system, program_id, event_routes_app::WASM_BINARY);

    // Send init message
    let header = SailsMessageHeader::v1(InterfaceId::zero(), 0, 0);
    _ = program.send_bytes(ACTOR_ID, header.encode());

    _ = system.run_next_block();

    // Send messages to services `Foo` and `Bar` to start `Foo`
    let header_foo = SailsMessageHeader::v1(event_routes_app::INTERFACE_ID, 0, 1);
    program.send_bytes(ACTOR_ID, header_foo.encode());
    let header_bar = SailsMessageHeader::v1(event_routes_app::INTERFACE_ID, 0, 2);
    program.send_bytes(ACTOR_ID, header_bar.encode());

    let run_result = system.run_next_block();

    // Ensure that both `Foo` and `Bar` have been started
    let header_foo_start = SailsMessageHeader::v1(event_routes_app::INTERFACE_ID, 1, 1);
    let log_foo_start = Log::builder()
        .source(program_id)
        .dest(0)
        .payload_bytes(header_foo_start.encode());

    let header_bar_start = SailsMessageHeader::v1(event_routes_app::INTERFACE_ID, 1, 2);
    let log_bar_start = Log::builder()
        .source(program_id)
        .dest(0)
        .payload_bytes(header_bar_start.encode());
    assert!(run_result.contains(&log_foo_start));
    assert!(run_result.contains(&log_bar_start));

    // // Send reply to message from `Foo` service
    let log = Log::builder().dest(ACTOR_ID).payload_bytes((1u8).encode());
    let _reply_id = system
        .get_mailbox(ACTOR_ID)
        .reply_bytes(log, [], 0)
        .unwrap();

    let run_result = system.run_next_block();

    // Ensure that `Foo` has been ended
    let header_foo_end = SailsMessageHeader::v1(event_routes_app::INTERFACE_ID, 0, 1);
    let log_foo_end = Log::builder()
        .source(program_id)
        .dest(0)
        .payload_bytes(header_foo_end.encode());
    assert!(run_result.contains(&log_foo_end));

    // Send reply to message from `Foo` service
    let log = Log::builder().dest(ACTOR_ID).payload_bytes((2u8).encode());
    let _reply_id = system
        .get_mailbox(ACTOR_ID)
        .reply_bytes(log, [], 0)
        .unwrap();

    let run_result = system.run_next_block();

    // Ensure that `Bar` has been ended
    let header_bar_end = SailsMessageHeader::v1(event_routes_app::INTERFACE_ID, 0, 2);
    let log_bar_end = Log::builder()
        .source(program_id)
        .dest(0)
        .payload_bytes(header_bar_end.encode());
    assert!(run_result.contains(&log_bar_end));
}
