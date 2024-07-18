use rmrk_resource_app::Program;
use std::{env, path::PathBuf};

fn main() {
    gwasm_builder::build();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("rmrk-resource.idl");

    sails_idl_gen::generate_idl_to_file::<Program>(&idl_file_path).unwrap();

    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let client_rs_file_path = out_dir_path.join("rmrk_resource.rs");
    sails_client_gen::generate_client_from_idl(idl_file_path, client_rs_file_path).unwrap();
}
