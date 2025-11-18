use std::{env, path::PathBuf};

use sails_build::{prepare_service_metadata, BuildScript, WasmBuildConfig};

fn service_paths() -> Vec<String> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    prepare_service_metadata(&out_dir)
        .map(|metadata| metadata.into_service_paths())
        .expect("failed to prepare service manifest")
}

fn main() {
    println!(
        "cargo:rerun-if-changed={}",
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("ping_pong_stack.idl")
            .display()
    );

    BuildScript::from_service_paths(service_paths())
        .manifest_path("Cargo.toml")
        .meta_dump_features(&["sails-canonical", "sails-meta-dump"])
        .wasm_build(WasmBuildConfig::new("CARGO_FEATURE_WASM_BUILDER", || {
            let _ = sails_rs::build_wasm();
        }))
        .before_emit(generate_client)
        .run()
        .unwrap_or_else(|err| panic!("failed to generate canonical interface constants: {err}"));
}

fn generate_client() {
    let base_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let idl_path = base_path.join("ping_pong_stack.idl");
    let out_path = base_path.join("src/ping_pong_stack.rs");
    sails_rs::ClientGenerator::from_idl_path(&idl_path)
        .generate_to(out_path)
        .unwrap();
}
