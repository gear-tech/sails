use rmrk_catalog_app::meta::ServiceMeta as RmrkCatalogServiceMeta;
use std::env;
use std::{fs::File, path::PathBuf};

fn main() {
    gwasm_builder::build();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("rmrk-catalog.idl");

    let idl_file = File::create(idl_file_path).unwrap();

    sails_idlgen::generate_serivce_idl::<RmrkCatalogServiceMeta>(idl_file).unwrap();
}
