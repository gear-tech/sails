#[allow(unused_imports)]
use no_svcs_prog::client::*;
use sails_rs::gtest::calls::GTestRemoting;

const ADMIN_ID: u64 = 10;
const WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/no_svcs_prog.opt.wasm";

#[test]
fn activating_program_succeeds() {
    let remoting = GTestRemoting::new(ADMIN_ID.into());
    remoting.system().init_logger();

    let _code_id = remoting.system().submit_code_file(WASM_PATH);

    // TODO: Activate via factory when client-gen is ready
}
