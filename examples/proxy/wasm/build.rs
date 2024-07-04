use proxy_app::ProxyProgram;
use sails_idl_gen::program;
use std::{env, fs::File, path::PathBuf};

fn main() {
    gwasm_builder::build();

    let idl_file =
        File::create(PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("proxy.idl"))
            .unwrap();

    program::generate_idl::<ProxyProgram>(idl_file).unwrap();
}
