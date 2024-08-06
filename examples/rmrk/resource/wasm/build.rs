use rmrk_resource_app::Program;
use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    sails_rs::build_wasm();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("rmrk-resource.idl");

    sails_idl_gen::generate_idl_to_file::<Program>(&idl_file_path).unwrap();

    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let client_rs_file_path = out_dir_path.join("rmrk_resource.rs");
    ClientGenerator::from_idl_path(&idl_file_path)
        .generate_to(client_rs_file_path)
        .unwrap();
}
