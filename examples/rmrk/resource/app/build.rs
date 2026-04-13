use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let client_rs_file_path = manifest_dir.join("src/rmrk_catalog.rs");
    let idl_file_path = manifest_dir.join("../../catalog/wasm/rmrk-catalog.idl");

    ClientGenerator::from_idl_path(&idl_file_path)
        .with_mocks("mockall")
        .generate_to(client_rs_file_path)
        .unwrap();
}
