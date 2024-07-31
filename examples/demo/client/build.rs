use demo::DemoProgram;
use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    let idl_file_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("demo.idl");

    // Generate IDL file for the `Demo` app
    sails_idl_gen::generate_idl_to_file::<DemoProgram>(&idl_file_path).unwrap();

    // Generate client code from IDL file
    ClientGenerator::from_idl_path(&idl_file_path)
        .with_mocks("with_mocks")
        .generate_to(PathBuf::from(env::var("OUT_DIR").unwrap()).join("demo_client.rs"))
        .unwrap();
}
