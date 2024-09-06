use sails_rs::{calls::*, gtest::calls::*};

use {{ client_crate_name }}::traits::*;

const ACTOR_ID: u64 = 42;

#[tokio::test]
async fn {{ service-name-snake }}_works() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

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
        .say_hello("World".to_string()) // Call service's method (see app/src/lib.rs:14)
        .send_recv(program_id)
        .await
        .unwrap();
    assert_eq!(result, "Hello World from SailsTest!".to_string());

    let result = service_client
        .last_name() // Call service's query (see app/src/lib.rs:19)
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(result, Some("World".to_string()));

    let _ = service_client
        .forget() // Call service's method (see app/src/lib.rs:14)
        .send_recv(program_id)
        .await
        .unwrap();

    let result = service_client
        .last_name() // Call service's query (see app/src/lib.rs:19)
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(result, None);
}
