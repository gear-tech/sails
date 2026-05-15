fn main() {
    sails_rs::build_wasm();

    // sails_rs::build_client::<rmrk_resource_app::Program>();

    use sails_rs::ClientGenerator;
    use std::{env, path::PathBuf};

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let idl_file_path = manifest_dir.join("rmrk_resource.idl");
    let client_rs_file_path = manifest_dir.join("src/rmrk_resource.rs");

    ClientGenerator::from_idl_path(&idl_file_path)
        .generate_to(client_rs_file_path)
        .unwrap();
}
