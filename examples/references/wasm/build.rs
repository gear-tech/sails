use references_app::ReferenceService;
use sails_idl_gen::service;
use std::{env, fs::File, path::PathBuf};

fn main() {
    gwasm_builder::build();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("references.idl");

    let idl_file = File::create(idl_file_path).unwrap();

    service::generate_idl::<ReferenceService>(idl_file).unwrap();
}
