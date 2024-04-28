use std::{env, path::PathBuf};

fn main() {
    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let idl_file_path = out_dir_path.join("rmrk-catalog.idl");

    let client_rs_file_path = out_dir_path.join("rmrk_catalog.rs");

    git_download::repo("https://github.com/gear-tech/sails")
        .branch_name("master")
        .add_file(
            "examples/rmrk/catalog/wasm/rmrk-catalog.idl",
            &idl_file_path,
        )
        .exec()
        .unwrap();

    sails_clientgen::generate_client_from_idl(&idl_file_path, client_rs_file_path).unwrap();
}
