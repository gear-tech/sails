use redirect_client::traits::{Redirect as _, RedirectFactory as _};
use redirect_proxy_client::traits::{Proxy as _, RedirectProxyFactory as _};
use sails_rs::{CodeId, GasUnit, calls::*};

const ACTOR_ID: u64 = 42;

#[tokio::test]
async fn redirect_on_exit_works() {
    let (remoting, program_code_id, proxy_code_id, _gas_limit) = create_remoting();

    let program_factory = redirect_client::RedirectFactory::new(remoting.clone());
    let proxy_factory = redirect_proxy_client::RedirectProxyFactory::new(remoting.clone());

    let program_id_1 = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"program_1")
        .await
        .unwrap();

    let program_id_2 = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"program_2")
        .await
        .unwrap();
    let program_id_3 = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"program_3")
        .await
        .unwrap();

    let proxy_program_id = proxy_factory
        .new(program_id_1) // Call program's constructor
        .send_recv(proxy_code_id, b"proxy")
        .await
        .unwrap();

    let mut redirect_client = redirect_client::Redirect::new(remoting.clone());
    let proxy_client = redirect_proxy_client::Proxy::new(remoting.clone());

    let result = proxy_client
        .get_program_id()
        .recv(proxy_program_id)
        .await
        .unwrap();

    assert_eq!(result, program_id_1);

    let _ = redirect_client
        .exit(program_id_2)
        .send(program_id_1)
        .await
        .unwrap();

    let result = proxy_client
        .get_program_id()
        .recv(proxy_program_id)
        .await
        .unwrap();

    assert_eq!(result, program_id_2);

    let _ = redirect_client
        .exit(program_id_3)
        .send(program_id_2)
        .await
        .unwrap();

    let result = proxy_client
        .get_program_id()
        .recv(proxy_program_id)
        .await
        .unwrap();

    assert_eq!(result, program_id_3);
}

fn create_remoting() -> (impl Remoting + Clone, CodeId, CodeId, GasUnit) {
    use sails_rs::gtest::{
        MAX_USER_GAS_LIMIT, System,
        calls::{BlockRunMode, GTestRemoting},
    };

    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug,redirect=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
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
