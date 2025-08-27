use no_svcs_prog::client::*;
use sails_rs::client::*;
use sails_rs::gtest::System;

const ADMIN_ID: u64 = 10;
const WASM_PATH: &str = "../../../target/wasm32-gear/debug/no_svcs_prog.opt.wasm";

#[tokio::test]
async fn activating_program_succeeds() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ADMIN_ID, 100_000_000_000_000);
    let program_code_id = system.submit_code_file(WASM_PATH);

    let env = GtestEnv::new(system, ADMIN_ID.into());

    let result = NoSvcsProgProgram::deploy(env, program_code_id, vec![])
        .create()
        .await;

    assert!(result.is_ok());
}
