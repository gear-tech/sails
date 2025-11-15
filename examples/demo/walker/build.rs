use std::{env, path::PathBuf};

macro_rules! sails_services {
    (
        $(type $alias:ident = $ty:ty;)*
        services: [
            $($path:path),* $(,)?
        ] $(,)?
    ) => {
        $(#[allow(dead_code)] pub type $alias = $ty;)*
        pub const SAILS_SERVICE_PATHS: &[&str] = &[$(stringify!($path)),*];
    };
    ($($path:path),* $(,)?) => {
        sails_services! {
            services: [ $($path),* ]
        }
    };
}

mod sails_services_manifest {
    include!("sails_services.in");
}

const SERVICE_PATHS: &[&str] = sails_services_manifest::SAILS_SERVICE_PATHS;

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=sails_services.in");

    if env::var_os("SAILS_CANONICAL_DUMP").is_some() {
        println!("cargo:rustc-cfg=sails_canonical_dump");
    }
    println!("cargo:rustc-check-cfg=cfg(sails_canonical_dump)");

    if should_skip_build_work() {
        return;
    }

    build_wasm_if_requested();

    if env::var_os("CARGO_FEATURE_SAILS_CANONICAL").is_none() {
        return;
    }

    emit_interface_consts();
}

fn should_skip_build_work() -> bool {
    env::var_os("SAILS_CANONICAL_DUMP").is_some()
        || env::var_os("CARGO_FEATURE_SAILS_META_DUMP").is_some()
}

fn build_wasm_if_requested() {
    if env::var_os("CARGO_FEATURE_WASM_BUILDER").is_some() {
        sails_rs::build_wasm();
    }
}

fn emit_interface_consts() {
    if SERVICE_PATHS.is_empty() {
        eprintln!("[sails-build] SERVICE_PATHS is empty; nothing to canonicalize");
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    sails_build::emit_interface_consts_with_options(
        "sails_rs",
        &["sails-canonical", "sails-meta-dump"],
        &out_dir,
    )
    .unwrap_or_else(|err| panic!("failed to generate canonical interface constants: {err}"));
}
