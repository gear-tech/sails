use rmrk_resource_app::services::meta::ServiceMeta as RmrkResourceServiceMeta;
use std::env;
use std::{fs::File, path::PathBuf};

fn main() {
    gwasm_builder::build();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("rmrk-resource.idl");

    let idl_file = File::create(idl_file_path).unwrap();

    sails_idlgen::generate_serivce_idl::<RmrkResourceServiceMeta>(idl_file).unwrap();
}
