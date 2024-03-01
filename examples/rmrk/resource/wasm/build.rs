use rmrk_resource_app::ResourceStorageService;
use sails_idlgen::service;
use std::{env, fs::File, path::PathBuf};

fn main() {
    gwasm_builder::build();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("rmrk-resource.idl");

    let idl_file = File::create(idl_file_path).unwrap();

    service::generate_idl::<ResourceStorageService>(idl_file).unwrap();
}
