use std::{env, path::PathBuf};

fn main() {
    sails_rs::build_wasm();

    sails_rs::generate_idl_to_file::<rmrk_catalog_app::Program>(
        Some("RmrkCatalog"),
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("rmrk-catalog.idl"),
    )
    .unwrap();
}
