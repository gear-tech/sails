fn main() {
    sails_rs::build_wasm();

    // sails_rs::build_client::<no_svcs_prog_app::Program>();

    use sails_client_gen::ClientGenerator;
    use std::{env, path::PathBuf};

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let idl_file_path = manifest_dir.join("no_svcs_prog.idl");
    let client_rs_file_path = manifest_dir.join("src/no_svcs_prog.rs");

    ClientGenerator::from_idl_path(&idl_file_path)
        .generate_to(client_rs_file_path)
        .unwrap();
}
