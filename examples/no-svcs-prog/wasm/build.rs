use no_svcs_prog_app::Program;
use std::{env, path::PathBuf};

fn main() {
    gwasm_builder::build();

    let idl_file_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("no-svcs-prog.idl");

    sails_idl_gen::generate_idl_to_file::<Program>(&idl_file_path).unwrap();

    sails_client_gen::generate_client_from_idl(
        &idl_file_path,
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("no_svcs_prog.rs"),
        None,
        None,
    )
    .unwrap();
}
