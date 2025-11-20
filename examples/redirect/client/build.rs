use sails_rs::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    // sails_rs::build_client::<redirect_app::RedirectProgram>();

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let idl_file_path = manifest_dir.join("redirect_client.idl");
    let client_rs_file_path = manifest_dir.join("src/redirect_client.rs");

    ClientGenerator::from_idl_path(&idl_file_path)
        .generate_to(client_rs_file_path)
        .unwrap();
}
