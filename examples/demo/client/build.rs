use sails_rs::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    // Generate IDL file for the `Demo` app and client code from IDL file
    // sails_rs::ClientBuilder::<demo::DemoProgram>::from_env().build_idl();
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let idl_file_path = manifest_dir.join("demo_client.idl");
    let client_rs_file_path = manifest_dir.join("src/demo_client.rs");

    ClientGenerator::from_idl_path(&idl_file_path)
        .with_mocks("with_mocks")
        .generate_to(client_rs_file_path)
        .unwrap();
}
