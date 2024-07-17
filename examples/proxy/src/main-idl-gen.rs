use proxy::ProxyProgram;
use sails_idl_gen::program;
use std::path::PathBuf;

fn main() {
    program::generate_idl_to_file::<ProxyProgram>(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proxy.idl"),
    )
    .unwrap();
}
