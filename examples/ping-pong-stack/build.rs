use std::{env, path::PathBuf};

use sails_build::{BuildScript, WasmBuildConfig};

macro_rules! sails_services_manifest {
    ($($tt:tt)*) => {
        sails_build::service_paths!($($tt)*)
    };
}

const SERVICE_PATHS: &[&str] = include!("sails_services.in");

fn main() {
    println!(
        "cargo:rerun-if-changed={}",
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("ping_pong_stack.idl")
            .display()
    );

    BuildScript::new(SERVICE_PATHS)
        .manifest_path("sails_services.in")
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
