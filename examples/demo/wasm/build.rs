use demo_app::DemoProgram;
use sails_idl_gen::program;
use std::{env, path::PathBuf};

fn main() {
    gwasm_builder::build();

    program::generate_idl_to_file::<DemoProgram>(
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("demo.idl"),
    )
    .unwrap();
}
