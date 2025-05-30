use demo::DemoProgram;
use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let idl_path = PathBuf::from(&out_dir).join("demo.idl");
    let client_path = PathBuf::from(&out_dir).join("src/demo_client.rs");

    // Generate IDL file for the `Demo` app
    sails_idl_gen::generate_idl_to_file::<DemoProgram>(&idl_path).unwrap();

    // Generate client code from IDL file
    ClientGenerator::from_idl_path(&idl_path)
        .with_mocks("with_mocks")
        .generate_to(&client_path)
        .unwrap();
}
