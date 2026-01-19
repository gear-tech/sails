use proxy::ProxyProgram;
use std::path::PathBuf;

fn main() {
    sails_rename::generate_idl_to_file::<ProxyProgram>(
        Some("ProxyProgram"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proxy.idl"),
    )
    .unwrap();
}
