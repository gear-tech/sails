use std::{env, path::PathBuf};

fn main() {
    sails::build_wasm();

    sails::generate_idl_to_file::<rmrk_catalog_app::Program>(
        Some("RmrkCatalog"),
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("rmrk-catalog.idl"),
    )
    .unwrap();
}
