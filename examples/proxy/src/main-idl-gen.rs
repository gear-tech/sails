use proxy::ProxyProgram;
use std::path::PathBuf;

fn main() {
    // TODO: Switch to new IDL generator crate
    /*
    sails_idl_gen::generate_idl_to_file::<ProxyProgram>(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proxy.idl"),
    )
    .unwrap();
    */
}
