use proxy_app::ProxyProgram;
use sails_idl_gen::program;
use std::{env, path::PathBuf};

fn main() {
    gwasm_builder::build();

    program::generate_idl_to_file::<ProxyProgram>(
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("proxy.idl"),
    )
    .unwrap();
}
