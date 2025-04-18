use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    let out_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let idl_file_path = out_dir_path.join("redirect_proxy.idl");

    // Generate IDL file for the program
    sails_rs::generate_idl_to_file::<redirect_proxy::ProxyProgram>(&idl_file_path).unwrap();

    // Generate client code from IDL file
    ClientGenerator::from_idl_path(&idl_file_path)
        .generate_to(
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("src/redirect_proxy_client.rs"),
        )
        .unwrap();
}
