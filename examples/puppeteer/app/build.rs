use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    let idl_path = Path::new("../../this-that-svc/wasm/this-that-svc.idl");

    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_dir_path.join("client.rs");
    sails_client_gen::generate_client_from_idl(idl_path, out_path).unwrap();
}
