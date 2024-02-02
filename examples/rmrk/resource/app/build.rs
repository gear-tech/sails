use std::{env, path::PathBuf};

fn main() {
    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let idl_file_path = out_dir_path.join("rmrk-catalog.idl");

    git_download::repo("https://github.com/gear-tech/sails")
        .branch_name("dd/rmrk-protocol")
        .add_file("examples/rmrk/catalog/wasm/rmrk-catalog.idl", idl_file_path)
        .exec()
        .unwrap();
}
