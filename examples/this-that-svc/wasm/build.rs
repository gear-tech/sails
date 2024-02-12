use std::env;
use std::{fs::File, path::PathBuf};
use this_that_svc_app::meta::ServiceMeta;

fn main() {
    gwasm_builder::build();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("this-that-svc.idl");

    let idl_file = File::create(idl_file_path).unwrap();

    sails_idlgen::generate_serivce_idl::<ServiceMeta>(idl_file).unwrap();
}
