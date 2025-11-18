use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

const BINPATH_FILE: &str = ".binpath";

fn main() {
    println!("cargo:rerun-if-changed=app/src");
    println!("cargo:rerun-if-changed=src");

    if should_skip_build_work() {
        return;
    }

    if env::var("__GEAR_WASM_BUILDER_NO_BUILD").is_ok() {
        return;
    }

    sails_rs::build_wasm();
    emit_idl_artifacts();
}

fn should_skip_build_work() -> bool {
    env::var_os("SAILS_CANONICAL_DUMP").is_some()
        || env::var_os("CARGO_FEATURE_SAILS_META_DUMP").is_some()
}

fn emit_idl_artifacts() {
    let mut bin_path_reader = BufReader::new(File::open(BINPATH_FILE).unwrap());
    let mut bin_path = String::new();
    bin_path_reader.read_line(&mut bin_path).unwrap();
    let mut idl_path = PathBuf::from(bin_path.trim());
    idl_path.set_extension("idl");
    sails_idl_gen::generate_idl_to_file::<{{ app_crate_name }}::{{ program-struct-name }}>(idl_path)
        .unwrap();
}
