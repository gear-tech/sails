use sails_rs::{calls::*, gtest::{calls::*, System}};

use {{ client_crate_name }}::traits::*;

const ACTOR_ID: u64 = 42;

#[tokio::test]
async fn do_something_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
    let remoting = GTestRemoting::new(system, ACTOR_ID.into());

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code({{ crate_name }}::WASM_BINARY);

    let program_factory = {{ client_crate_name }}::{{ service-name }}Factory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor (see app/src/lib.rs:29)
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = {{ client_crate_name }}::{{ service-name }}::new(remoting.clone());

    let result = service_client
        .do_something() // Call service's method (see app/src/lib.rs:14)
        .send_recv(program_id)
        .await
        .unwrap();

    assert_eq!(result, "Hello from {{ service-name }}!".to_string());
}

#[tokio::test]
async fn get_something_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
    let remoting = GTestRemoting::new(system, ACTOR_ID.into());

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code({{ crate_name }}::WASM_BINARY);

    let program_factory = {{ client_crate_name }}::{{ service-name }}Factory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor (see app/src/lib.rs:29)
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let service_client = {{ client_crate_name }}::{{ service-name }}::new(remoting.clone());

    let result = service_client
        .get_something() // Call service's query (see app/src/lib.rs:19)
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(result, "Hello from {{ service-name }}!".to_string());
}
