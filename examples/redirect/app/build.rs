use std::{env, path::PathBuf};

macro_rules! sails_services {
    ($($path:path),* $(,)?) => {
        &[$(stringify!($path)),*]
    };
}

const SERVICE_PATHS: &[&str] = include!("sails_services.in");

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=sails_services.in");

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
    for service in SERVICE_PATHS {
        sails_build::emit_interface_consts_with_options(
            service,
            "sails_rs",
            &["sails-canonical", "sails-meta-dump"],
            &out_dir,
        )
        .unwrap_or_else(|err| panic!("failed to generate canonical interface constants for {service}: {err}"));
    }
}
