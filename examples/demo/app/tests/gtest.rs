use demo_client::*;
use sails_rs::{
    client::*,
    futures::StreamExt as _,
    gtest::{Program, System},
    prelude::*,
};

const ACTOR_ID: u64 = 42;
#[cfg(debug_assertions)]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/debug/demo.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/release/demo.opt.wasm";

fn create_env() -> (GtestEnv, CodeId, GasUnit) {
    use sails_rs::gtest::{System, constants::MAX_USER_GAS_LIMIT};

    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug,redirect=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
    // Submit program code into the system
    let code_id = system.submit_code_file(DEMO_WASM_PATH);

    // Create a remoting instance for the system
    // and set the block run mode to Next,
    // cause we don't receive any reply on `Exit` call
    let env = GtestEnv::new(system, ACTOR_ID.into());
    (env, code_id, MAX_USER_GAS_LIMIT)
}

#[tokio::test]
async fn counter_add_works() {
    use demo_client::counter::{Counter as _, events::CounterEvents};
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    // Use generated client code for activating Demo program
    // using the `new` constructor
    let demo_program = env
        .deploy(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let mut counter_client = demo_program.counter();
    // Listen to Counter events
    let counter_listener = counter_client.listener();
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Act

    // Use generated client code for calling Counter service
    // using the `send_recv` method
    let result = counter_client.add(10).await.unwrap();

    // Assert
    let event = counter_events.next().await.unwrap();

    assert_eq!(result, 52);
    assert_eq!((demo_program.id(), CounterEvents::Added(10)), event);
}

#[tokio::test]
async fn counter_sub_works() {
    use demo_client::counter::{Counter as _, events::CounterEvents};
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    // Use generated client code for activating Demo program
    // using the `new` constructor
    let demo_program = env
        .deploy(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let mut counter_client = demo_program.counter();
    // Listen to Counter events
    let counter_listener = counter_client.listener();
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Act

    // Use generated client code for calling Counter service
    // using the `send`/`recv` pair of methods
    let result = counter_client.sub(10).await.unwrap();

    // Assert
    let event = counter_events.next().await.unwrap();

    assert_eq!(result, 32);
    assert_eq!((demo_program.id(), CounterEvents::Subtracted(10)), event);
}

#[tokio::test]
async fn counter_query_works() {
    use demo_client::counter::Counter as _;
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send`/`recv` pair
    // of methods
    let demo_program = env
        .deploy(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let counter_client = demo_program.counter();

    // Act

    // Use generated client code for query Counter service using the `recv` method
    let result = counter_client.value().await.unwrap();

    // Assert
    assert_eq!(result, 42);
}

#[tokio::test]
async fn counter_query_not_enough_gas() {
    use demo_client::counter::Counter as _;
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send`/`recv` pair
    // of methods
    let demo_program = env
        .deploy(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let counter_client = demo_program.counter();

    // Act

    // Use generated client code for query Counter service using the `recv` method
    let result = counter_client
        .value()
        .with_gas_limit(0) // Set gas_limit to 0
        .await;

    // Assert
    println!("{result:?}");
    assert!(matches!(
        result,
        Err(GtestError::ReplyHasError(
            ErrorReplyReason::Execution(SimpleExecutionError::RanOutOfGas),
            _
        ))
    ));
}

/// Low level program test using `gtest::System` and call encoding/decoding with `io` module
#[tokio::test]
async fn ping_pong_low_level_works() {
    use demo_client::{io::Default, ping_pong::io::Ping};

    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ACTOR_ID, 1_000_000_000_000_000);

    let demo_program = Program::from_file(&system, DEMO_WASM_PATH);

    // Use generated `io` module to create a program
    let message_id = demo_program.send_bytes(ACTOR_ID, Default::encode_params());
    let run_result = system.run_next_block();
    let gas_burned = *run_result
        .gas_burned
        .get(&message_id)
        .expect("message not found");
    let wasm_size = std::fs::metadata(DEMO_WASM_PATH).unwrap().len();
    println!("[ping_pong_low_level_works] Init Gas: {gas_burned:>14}, Size: {wasm_size}");

    // Use generated `io` module for encoding/decoding calls and replies
    // and send/receive bytes using `gtest` native means
    let ping_call_payload = Ping::encode_params_with_prefix("PingPong", "ping".into());

    let message_id = demo_program.send_bytes(ACTOR_ID, ping_call_payload);
    let run_result = system.run_next_block();

    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(message_id))
        .unwrap();

    let ping_reply_payload = reply_log_record.payload();

    let ping_reply = Ping::decode_reply_with_prefix("PingPong", ping_reply_payload).unwrap();

    assert_eq!(ping_reply, Ok("pong".to_string()));

    let gas_burned = *run_result
        .gas_burned
        .get(&message_id)
        .expect("message not found");
    println!("[ping_pong_low_level_works] Handle Gas: {gas_burned:>14}, Size: {wasm_size}");
}

#[tokio::test]
async fn dog_barks() {
    use demo_client::dog::{Dog as _, events::DogEvents};
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send`/`recv` pair
    // of methods
    let demo_program = env
        .deploy(code_id, vec![])
        .new(None, Some((1, -1)))
        .await
        .unwrap();

    let mut dog_client = demo_program.dog();
    let dog_listener = dog_client.listener();
    let mut dog_events = dog_listener.listen().await.unwrap();

    // Act
    let result = dog_client.make_sound().await.unwrap();

    // Assert
    let event = dog_events.next().await.unwrap();

    assert_eq!(result, "Woof! Woof!");
    assert_eq!((demo_program.id(), DogEvents::Barked), event);
}

#[tokio::test]
async fn dog_walks() {
    use demo_client::dog::{Dog as _, events::DogEvents};
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send`/`recv` pair
    // of methods
    let demo_program = env
        .deploy(code_id, vec![])
        .new(None, Some((1, -1)))
        .await
        .unwrap();

    let mut dog_client = demo_program.dog();
    let dog_listener = dog_client.listener();
    let mut dog_events = dog_listener.listen().await.unwrap();

    // Act
    dog_client.walk(10, 20).await.unwrap();

    // Assert
    let position = dog_client.position().await.unwrap();
    let event = dog_events.next().await.unwrap();

    assert_eq!(position, (11, 19));
    assert_eq!(
        (
            demo_program.id(),
            DogEvents::Walked {
                from: (1, -1),
                to: (11, 19)
            }
        ),
        event
    );
}

#[tokio::test]
async fn dog_weights() {
    use demo_client::dog::Dog as _;
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    // Use generated client code for activating Demo program
    // using the `new` constructor and the `send`/`recv` pair
    // of methods
    let demo_program = env
        .deploy(code_id, vec![])
        .new(None, Some((1, -1)))
        .await
        .unwrap();

    let dog_client = demo_program.dog();

    let avg_weight = dog_client.avg_weight().await.unwrap();

    assert_eq!(avg_weight, 42);
}

#[tokio::test]
async fn references_add() {
    use demo_client::references::References as _;
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    let demo_program = env
        .deploy(code_id, vec![])
        .new(None, Some((1, -1)))
        .await
        .unwrap();

    let mut client = demo_program.references();

    let value = client.add(42).await.unwrap();

    assert_eq!(42, value);
}

#[tokio::test]
async fn references_bytes() {
    use demo_client::references::References as _;
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    let demo_program = env
        .deploy(code_id, vec![])
        .new(None, Some((1, -1)))
        .await
        .unwrap();

    let mut client = demo_program.references();

    _ = client.add_byte(42).await.unwrap();
    _ = client.add_byte(89).await.unwrap();
    _ = client.add_byte(14).await.unwrap();

    let last = client.last_byte().await.unwrap();
    assert_eq!(Some(14), last);
}

#[tokio::test]
async fn references_guess_num() {
    use demo_client::references::References as _;
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    let demo_program = env
        .deploy(code_id, vec![])
        .new(None, Some((1, -1)))
        .await
        .unwrap();

    let mut client = demo_program.references();

    let res1 = client.guess_num(42).await.unwrap();
    let res2 = client.guess_num(89).await.unwrap();
    let res3 = client.message().await.unwrap();
    let res4 = client.set_num(14).await.unwrap();
    let res5 = client.guess_num(14).await.unwrap();

    assert_eq!(Ok("demo".to_owned()), res1);
    assert_eq!(Err("Number is too large".to_owned()), res2);
    assert_eq!(Some("demo".to_owned()), res3);
    assert_eq!(Ok(()), res4);
    assert_eq!(Ok("demo".to_owned()), res5);
}

#[tokio::test]
async fn counter_add_works_via_next_mode() {
    use demo_client::counter::{Counter as _, events::CounterEvents};
    // Arrange
    let (env, code_id, _gas_limit) = create_env();
    let env = env.with_block_run_mode(BlockRunMode::Next);

    let demo_program = env
        .deploy(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let mut counter_client = demo_program.counter();
    // Listen to Counter events
    let counter_listener = counter_client.listener();
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Act
    let result = counter_client.add(10).await.unwrap();

    // Assert
    assert_eq!(result, 52);
    assert_eq!(
        (demo_program.id(), CounterEvents::Added(10)),
        counter_events.next().await.unwrap()
    );
}

#[tokio::test]
async fn counter_add_works_via_manual_mode() {
    use demo_client::counter::{Counter as _, events::CounterEvents};
    // Arrange
    let (env, code_id, _gas_limit) = create_env();
    let env = env.with_block_run_mode(BlockRunMode::Next);

    let pending_ctor = env
        .deploy(code_id, vec![])
        .new(Some(42), None)
        .create_program()
        .unwrap();

    // Run next Block
    env.run_next_block();

    let demo_program = pending_ctor.await.unwrap();

    let mut counter_client = demo_program.counter();
    // Listen to Counter events
    let counter_listener = counter_client.listener();
    let mut counter_events = counter_listener.listen().await.unwrap();

    // Use generated client code for calling Counter service
    let call_add = counter_client.add(10).send_for_reply().unwrap();
    let call_sub = counter_client.sub(20).send_for_reply().unwrap();

    // Run next Block
    env.run_next_block();

    // Got replies
    let result_add = call_add.await.unwrap();
    assert_eq!(result_add, 52);
    let result_sub = call_sub.await.unwrap();
    assert_eq!(result_sub, 32);

    // Got events
    assert_eq!(
        (demo_program.id(), CounterEvents::Added(10)),
        counter_events.next().await.unwrap()
    );
    assert_eq!(
        (demo_program.id(), CounterEvents::Subtracted(20)),
        counter_events.next().await.unwrap()
    );
}

#[test]
fn counter_add_low_level_works() {
    use demo_client::{counter::io::Add, io::Default};

    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ACTOR_ID, 1_000_000_000_000_000);

    let demo_program = Program::from_file(&system, DEMO_WASM_PATH);
    let wasm_size = std::fs::metadata(DEMO_WASM_PATH).unwrap().len();

    // Use generated `io` module to create a program
    demo_program.send_bytes(ACTOR_ID, Default::encode_params());

    // Use generated `io` module for encoding/decoding calls and replies
    // and send/receive bytes using `gtest` native means
    let call_payload = Add::encode_params_with_prefix("Counter", 10);

    let message_id = demo_program.send_bytes(ACTOR_ID, call_payload);
    let run_result = system.run_next_block();

    let reply_log_record = run_result
        .log()
        .iter()
        .find(|entry| entry.reply_to() == Some(message_id))
        .unwrap();

    let reply_payload = reply_log_record.payload();

    let reply = Add::decode_reply_with_prefix("Counter", reply_payload).unwrap();

    assert_eq!(reply, 10);

    let gas_burned = *run_result
        .gas_burned
        .get(&message_id)
        .expect("message not found");
    println!("[counter_add_low_level_works] Handle Gas: {gas_burned:>14}, Size: {wasm_size}");
}

#[tokio::test]
async fn value_fee_works() {
    use demo_client::value_fee::ValueFee as _;
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    let demo_program = env
        .deploy(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let initial_balance = env.system().balance_of(ActorId::from(ACTOR_ID));
    let mut client = demo_program.value_fee();

    // Act

    // Use generated client code to call `do_something_and_take_fee` method with zero value
    let result = client.do_something_and_take_fee().await.unwrap();
    assert!(!result);

    // Use generated client code to call `do_something_and_take_fee` method with value
    let result = client
        .do_something_and_take_fee()
        .with_value(15_000_000_000_000)
        .await
        .unwrap();

    assert!(result);
    let balance = env.system().balance_of(ActorId::from(ACTOR_ID));
    // fee is 10_000_000_000_000 + spent gas
    // initial_balance - balance = 10_329_809_407_200
    assert!(
        initial_balance - balance > 10_000_000_000_000
            && initial_balance - balance < 10_500_000_000_000
    );
}

#[tokio::test]
async fn program_value_transfer_works() {
    // Arrange
    let (env, code_id, _gas_limit) = create_env();

    let demo_program = env
        .deploy(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();
    let program_id = demo_program.id();

    let initial_balance = env.system().balance_of(program_id);

    // Act
    // send empty bytes with value 1_000_000_000_000 to the program
    _ = env
        .system()
        .get_program(program_id)
        .map(|prg| prg.send_bytes_with_value(ACTOR_ID, Vec::<u8>::new(), 1_000_000_000_000));

    env.run_next_block();

    // Assert
    let balance = env.system().balance_of(program_id);
    assert_eq!(initial_balance + 1_000_000_000_000, balance);
}

#[test]
fn chaos_service_panic_after_wait_works() {
    use demo_client::io::Default;
    use gstd::errors::{ErrorReplyReason, SimpleExecutionError};
    use sails_rs::gtest::{Log, Program, System};

    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 1_000_000_000_000_000);
    let program = Program::from_file(&system, DEMO_WASM_PATH);
    program.send_bytes(ACTOR_ID, Default::encode_params());

    let msg_id = program.send_bytes(ACTOR_ID, ("Chaos", "PanicAfterWait").encode());
    system.run_next_block();

    let log = Log::builder().source(program.id()).dest(ACTOR_ID);
    system
        .get_mailbox(ACTOR_ID)
        .reply_bytes(log.clone().payload_bytes(().encode()), vec![], 0)
        .unwrap();

    let run_result = system.run_next_block();

    assert!(
        run_result.contains(&log.reply_to(msg_id).reply_code(ReplyCode::Error(
            ErrorReplyReason::Execution(SimpleExecutionError::UserspacePanic)
        )))
    );
}

#[test]
fn chaos_service_timeout_wait() {
    use demo_client::chaos::io::ReplyHookCounter;
    use demo_client::io::Default;
    use sails_rs::gtest::{Log, Program, System};

    fn extract_reply<T, F>(run: &gtest::BlockRunResult, msg_id: MessageId, decode: F) -> T
    where
        F: FnOnce(&[u8]) -> T,
    {
        let payload = run
            .log()
            .iter()
            .find_map(|log| {
                log.reply_to()
                    .filter(|&r| r == msg_id)
                    .map(|_| log.payload())
            })
            .expect("reply not found");
        decode(payload)
    }

    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug,redirect=debug");
    system.mint_to(ACTOR_ID, 1_000_000_000_000_000);
    let program = Program::from_file(&system, DEMO_WASM_PATH);
    program.send_bytes(ACTOR_ID, Default::encode_params());

    program.send_bytes(ACTOR_ID, ("Chaos", "TimeoutWait").encode());
    //#1
    system.run_next_block();
    //#2
    system.run_next_block();

    let msg_id = program.send_bytes(ACTOR_ID, ("Chaos", "ReplyHookCounter").encode());

    let run = system.run_next_block();

    let val = extract_reply(&run, msg_id, |p| {
        ReplyHookCounter::decode_reply_with_prefix("Chaos", p).unwrap()
    });
    assert_eq!(val, 0, "handle_reply should not trigger before reply");

    let log = Log::builder().source(program.id()).dest(ACTOR_ID);
    system
        .get_mailbox(ACTOR_ID)
        .reply_bytes(log.payload_bytes(().encode()), vec![], 0)
        .unwrap();
    system.run_next_block();

    let msg_id = program.send_bytes(ACTOR_ID, ("Chaos", "ReplyHookCounter").encode());
    let run = system.run_next_block();
    let val = extract_reply(&run, msg_id, |p| {
        ReplyHookCounter::decode_reply_with_prefix("Chaos", p).unwrap()
    });
    assert_eq!(
        val, 1,
        "handle_reply should still execute even after timeout if reply arrives"
    );
}

#[tokio::test]
async fn chaos_panic_does_not_affect_other_services() {
    use demo_client::chaos::Chaos as _;
    use demo_client::counter::Counter as _;

    const INIT_VALUE: u32 = 100;

    let (env, code_id, _gas_limit) = create_env();
    let demo_program = env
        .deploy(code_id, vec![])
        .new(Some(INIT_VALUE), None)
        .await
        .unwrap();

    let mut counter_client = demo_program.counter();
    let chaos_client = demo_program.chaos();

    let initial_value = counter_client.value().await.unwrap();
    assert_eq!(initial_value, INIT_VALUE);

    let panic_result = chaos_client.panic_after_wait().await;

    assert!(matches!(
        panic_result,
        Err(GtestError::ReplyHasError(
            ErrorReplyReason::Execution(SimpleExecutionError::UserspacePanic),
            _
        ))
    ));

    counter_client.add(5).await.unwrap();

    let final_value = counter_client.value().await.unwrap();
    assert_eq!(final_value, INIT_VALUE + 5);
}
