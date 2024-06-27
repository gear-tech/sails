use sails_idl_gen::service;
use std::{env, fs::File, path::PathBuf};
use this_that_svc_app::MyService;

fn main() {
    gwasm_builder::build();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("this-that-svc.idl");

    let idl_file = File::create(idl_file_path).unwrap();

    service::generate_idl::<MyService>(idl_file).unwrap();
}
