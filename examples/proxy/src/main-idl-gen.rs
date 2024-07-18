use proxy::ProxyProgram;
use std::path::PathBuf;

fn main() {
    sails_idl_gen::generate_idl_to_file::<ProxyProgram>(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proxy.idl"),
    )
    .unwrap();
}
