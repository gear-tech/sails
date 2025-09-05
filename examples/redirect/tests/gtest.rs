use redirect_client::{redirect::Redirect as _, *};
use redirect_proxy_client::{proxy::Proxy as _, *};
use sails_rs::{CodeId, GasUnit, client::*};

const ACTOR_ID: u64 = 42;

#[tokio::test]
async fn redirect_on_exit_works() {
    let (env, program_code_id, proxy_code_id, _gas_limit) = create_env();

    let program_factory_1 = env.deploy::<RedirectClientProgram>(program_code_id, vec![1]);
    let program_factory_2 = env.deploy::<RedirectClientProgram>(program_code_id, vec![2]);
    let program_factory_3 = env.deploy::<RedirectClientProgram>(program_code_id, vec![3]);
    let proxy_factory = env.deploy::<RedirectProxyClientProgram>(proxy_code_id, vec![]);

    let program_1 = program_factory_1
        .new() // Call program's constructor
        .await
        .unwrap();

    let program_2 = program_factory_2
        .new() // Call program's constructor
        .await
        .unwrap();
    let program_3 = program_factory_3
        .new() // Call program's constructor
        .await
        .unwrap();

    let proxy_program = proxy_factory
        .new(program_1.id()) // Call program's constructor
        .await
        .unwrap();

    let result = proxy_program.proxy().get_program_id().await.unwrap();
    assert_eq!(result, program_1.id());

    program_1.redirect().exit(program_2.id()).await.unwrap();

    let result = proxy_program.proxy().get_program_id().await.unwrap();
    assert_eq!(result, program_2.id());

    program_2.redirect().exit(program_3.id()).await.unwrap();

    let result = proxy_program.proxy().get_program_id().await.unwrap();
    assert_eq!(result, program_3.id());
}

fn create_env() -> (GtestEnv, CodeId, CodeId, GasUnit) {
    use sails_rs::gtest::{MAX_USER_GAS_LIMIT, System};

    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug,redirect=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
    // Submit program code into the system
    let program_code_id = system.submit_code(redirect_app::WASM_BINARY);
    let proxy_code_id = system.submit_code(redirect_proxy::WASM_BINARY);

    // Create a remoting instance for the system
    // and set the block run mode to Next,
    // cause we don't receive any reply on `Exit` call
    let env = GtestEnv::new(system, ACTOR_ID.into()).with_block_run_mode(BlockRunMode::Next);
    (env, program_code_id, proxy_code_id, MAX_USER_GAS_LIMIT)
}
