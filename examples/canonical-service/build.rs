use sails_build::{prepare_service_metadata, BuildScript, WasmBuildConfig};
use std::{env, path::PathBuf};

fn service_paths() -> Vec<String> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    prepare_service_metadata(&out_dir)
        .map(|metadata| metadata.into_service_paths())
        .expect("failed to prepare service manifest")
}

fn main() {
    BuildScript::from_service_paths(service_paths())
        .manifest_path("Cargo.toml")
        .meta_dump_features(&["sails-canonical", "sails-meta-dump"])
        .wasm_build(WasmBuildConfig::new("CARGO_FEATURE_WASM_BUILDER", || {
            let _ = sails_rs::build_wasm();
        }))
        .run()
        .unwrap_or_else(|err| panic!("failed to generate canonical interface constants: {err}"));
}
