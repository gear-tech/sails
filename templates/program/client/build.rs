use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

use {{ crate_name }}::{{ program-struct-name }} as ProgramType;

fn main() {
    let idl_file_path = PathBuf::from("{{ crate_name }}.idl");
    // Generate IDL file for the program
    sails_idl_gen::generate_idl_to_file::<ProgramType>(&idl_file_path).unwrap();
    // Generate client code from IDL file
    ClientGenerator::from_idl_path(&idl_file_path)
        .with_mocks("{{ mocks-feature-name}}")
        .generate_to(PathBuf::from(env::var("OUT_DIR").unwrap()).join("{{ client_crate_name }}.rs"))
        .unwrap();
}
