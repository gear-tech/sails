use sails_rs::ClientGenerator;
use std::path::Path;

fn main() {
    let idl_path = Path::new("partial_idl_client.idl");
    let client_path = Path::new("src/partial_idl_client.rs");

    ClientGenerator::from_idl_path(&idl_path)
        .with_client_path(&client_path)
        .generate()
        .unwrap();
}
