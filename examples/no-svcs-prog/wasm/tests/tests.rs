use no_svcs_prog::client::{traits::NoSvcsProgFactory as NoSvcsProgFactoryTrait, *};
use sails_rs::{calls::*, gtest::calls::*};

const ADMIN_ID: u64 = 10;
const WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/no_svcs_prog.opt.wasm";

#[tokio::test]
async fn activating_program_succeeds() {
    let remoting = GTestRemoting::new(ADMIN_ID.into());
    remoting.system().init_logger();
    remoting.system().mint_to(ADMIN_ID, 10_000_000_000_000);

    let program_code_id = remoting.system().submit_code_file(WASM_PATH);

    let program_id = NoSvcsProgFactory::new(remoting.clone())
        .default()
        .send_recv(program_code_id, "123")
        .await
        .unwrap();

    assert!(remoting.system().get_program(program_id.as_ref()).is_some());
}
