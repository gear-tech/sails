use ping_pong_stack::client::{
    PingPongStack, PingPongStackCtors, PingPongStackProgram, ping_pong_stack::PingPongStack as _,
};
use sails_rs::{CodeId, GasUnit, client::*};

const ACTOR_ID: u64 = 42;

#[tokio::test]
async fn ping_pong_stack_works() {
    let (env, code_id, _gas_limit) = create_env();

    let program_1 = env
        .deploy::<PingPongStackProgram>(code_id, vec![1])
        .new_for_bench()
        .await
        .unwrap();

    let program_2 = env
        .deploy::<PingPongStackProgram>(code_id, vec![2])
        .new_for_bench()
        .await
        .unwrap();

    let limit = 10;
    let initial_balance = env.system().balance_of(ACTOR_ID);

    program_1
        .ping_pong_stack()
        .start(program_2.id(), limit)
        .await
        .unwrap();

    let balance = env.system().balance_of(ACTOR_ID);

    println!(
        "[ping_pong_stack_works] limit: {:02}, burned: {:>14}",
        limit,
        initial_balance - balance,
    );
}

fn create_env() -> (GtestEnv, CodeId, GasUnit) {
    use sails_rs::gtest::{System, constants::MAX_USER_GAS_LIMIT};

    let system = System::new();
    system.init_logger_with_default_filter(
        "gwasm=debug,gtest=info,sails_rs=debug,ping_pong_stack=debug",
    );
    system.mint_to(ACTOR_ID, 1_000_000_000_000_000);
    // Submit program code into the system
    let code_id = system.submit_code(ping_pong_stack::WASM_BINARY);

    // Create a remoting instance for the system
    // and set the block run mode to Next,
    // cause we don't receive any reply on `Exit` call
    let env = GtestEnv::new(system, ACTOR_ID.into());
    (env, code_id, MAX_USER_GAS_LIMIT)
}
