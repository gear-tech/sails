use sails_build::{BuildScript, WasmBuildConfig};

macro_rules! sails_services_manifest {
    ($($tt:tt)*) => {
        sails_build::service_paths!($($tt)*)
    };
}

const SERVICE_PATHS: &[&str] = include!("sails_services.in");

fn main() {
    BuildScript::new(SERVICE_PATHS)
        .manifest_path("sails_services.in")
        .sails_crate_path("sails_rename")
        .meta_dump_features(&["sails-canonical", "sails-meta-dump"])
        .wasm_build(WasmBuildConfig::new("CARGO_FEATURE_WASM_BUILDER", || {
            let _ = sails_rename::build_wasm();
        }))
        .run()
        .unwrap_or_else(|err| panic!("failed to generate canonical interface constants: {err}"));
}
