use no_svcs_prog::client::{traits::NoSvcsProgFactory as NoSvcsProgFactoryTrait, *};
use sails_rs::{
    calls::*,
    gtest::{calls::*, System},
};

const ADMIN_ID: u64 = 10;
const WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/no_svcs_prog.opt.wasm";

#[tokio::test]
async fn activating_program_succeeds() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ADMIN_ID, 100_000_000_000_000);
    let program_code_id = system.submit_code_file(WASM_PATH);

    let remoting = GTestRemoting::new(system, ADMIN_ID.into());

    let result = NoSvcsProgFactory::new(remoting.clone())
        .default()
        .send_recv(program_code_id, "123")
        .await;

    assert!(result.is_ok());
}
