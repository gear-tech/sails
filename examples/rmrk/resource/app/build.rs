use sails_build::{prepare_service_metadata, BuildScript, WasmBuildConfig};
use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn service_paths() -> Vec<String> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    prepare_service_metadata(&out_dir)
        .map(|metadata| metadata.into_service_paths())
        .expect("failed to prepare service manifest")
}

fn main() {
    let wasm_config = WasmBuildConfig::new("CARGO_FEATURE_WASM_BUILDER", || {
        let _ = sails_rs::build_wasm();
    })
    .skip_features(&["CARGO_FEATURE_MOCKALL"])
    .skip_env(&["__GEAR_WASM_BUILDER_NO_BUILD"]);

    if env::var_os("CARGO_FEATURE_MOCKALL").is_some() {
        eprintln!("[sails-build] mockall feature enabled; skipping wasm build for host-only tests");
    }

    BuildScript::from_service_paths(service_paths())
        .manifest_path("Cargo.toml")
        .meta_dump_features(&["sails-canonical", "sails-meta-dump"])
        .wasm_build(wasm_config)
        .run()
        .unwrap_or_else(|err| panic!("failed to generate canonical interface constants: {err}"));

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let client_rs_file_path = manifest_dir.join("src/rmrk_catalog.rs");

    #[cfg(not(target_family = "windows"))]
    let idl_file_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("rmrk-catalog.idl");
    #[cfg(not(target_family = "windows"))]
    git_download::repo("https://github.com/gear-tech/sails")
        .branch_name("master")
        .add_file(
            "examples/rmrk/catalog/wasm/rmrk-catalog.idl",
            &idl_file_path,
        )
        .exec()
        .unwrap();

    #[cfg(target_family = "windows")]
    let idl_file_path = manifest_dir.join("..\\..\\catalog\\wasm\\rmrk-catalog.idl");

    ClientGenerator::from_idl_path(&idl_file_path)
        .with_mocks("mockall")
        .generate_to(client_rs_file_path)
        .unwrap();
}
