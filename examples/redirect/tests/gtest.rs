use redirect_client::{traits::{RedirectClient as _, RedirectClientFactory as _}, Redirect};
use redirect_proxy_client::traits::{Proxy as _, RedirectProxyClientFactory as _};
use sails_rs::{CodeId, GasUnit, calls::*, gtest::calls::GTestRemoting};

const ACTOR_ID: u64 = 42;

#[tokio::test]
async fn redirect_on_exit_works() {
    let (remoting, program_code_id, proxy_code_id, _gas_limit) = create_remoting();

    // let program_factory = ProgramFactory::new(remoting.clone());
    let proxy_factory = redirect_proxy_client::RedirectProxyClientFactory::new(remoting.clone());

    let mut program_1 = remoting
        .new() // Call program's constructor
        .deploy(program_code_id, b"program_1")
        .await
        .unwrap();
    let program_id_1 = program_1.program_id();

    let mut program_2: redirect_client::Redirect<GTestRemoting> = remoting
        .new() // Call program's constructor
        .deploy(program_code_id, b"program_2")
        .await
        .unwrap();
    let program_id_2 = program_2.program_id();

    let program_3 = remoting
        .new() // Call program's constructor
        .deploy(program_code_id, b"program_3")
        .await
        .unwrap();

    let proxy_program_id = proxy_factory
        .new(program_id_1) // Call program's constructor
        .send_recv(proxy_code_id, b"proxy")
        .await
        .unwrap();

    // let mut redirect_client = redirect_client::Redirect::new(remoting.clone(), program_1.program_id());
    let proxy_client = redirect_proxy_client::Proxy::new(remoting.clone());

    let result = proxy_client
        .get_program_id()
        .recv(proxy_program_id)
        .await
        .unwrap();

    assert_eq!(result, program_id_1);

    let p1 = program_1.get_program_id().await.unwrap();
    assert_eq!(p1, program_id_1);

    let _ = program_1.exit(program_id_2).send_one_way().unwrap();

    remoting.run_next_block();

    let result = proxy_client
        .get_program_id()
        .recv(proxy_program_id)
        .await
        .unwrap();

    assert_eq!(result, program_id_2);

    let _ = program_2
        .exit(program_3.program_id())
        .send_one_way()
        .unwrap();

    remoting.run_next_block();

    let result = proxy_client
        .get_program_id()
        .recv(proxy_program_id)
        .await
        .unwrap();

    assert_eq!(result, program_3.program_id());
}

fn create_remoting() -> (GTestRemoting, CodeId, CodeId, GasUnit) {
    use sails_rs::gtest::{
        MAX_USER_GAS_LIMIT, System,
        calls::{BlockRunMode, GTestRemoting},
    };

    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug,redirect=debug");
    system.mint_to(ACTOR_ID, 1_000_000_000_000_000);
    // Submit program code into the system
    let program_code_id = system.submit_code(redirect_app::WASM_BINARY);
    let proxy_code_id = system.submit_code(redirect_proxy::WASM_BINARY);

    // Create a remoting instance for the system
    // and set the block run mode to Next,
    // cause we don't receive any reply on `Exit` call
    let remoting =
        GTestRemoting::new(system, ACTOR_ID.into()).with_block_run_mode(BlockRunMode::Next);
    (remoting, program_code_id, proxy_code_id, MAX_USER_GAS_LIMIT)
}
