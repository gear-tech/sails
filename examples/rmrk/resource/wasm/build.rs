use rmrk_resource_app::Program;
use sails_idl_gen::program;
use std::{env, fs::File, path::PathBuf};

fn main() {
    gwasm_builder::build();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("rmrk-resource.idl");

    let idl_file = File::create(idl_file_path.clone()).unwrap();

    program::generate_idl::<Program>(idl_file).unwrap();

    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let client_rs_file_path = out_dir_path.join("rmrk_resource.rs");
    sails_client_gen::generate_client_from_idl(idl_file_path, client_rs_file_path).unwrap();
}
