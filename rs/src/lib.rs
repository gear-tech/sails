#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![no_std]

#[cfg(feature = "std")]
#[cfg(not(target_arch = "wasm32"))]
extern crate std;

#[cfg(feature = "client-builder")]
pub use builder::{ClientBuilder, ClientGenerator, IdlPath, build_client, build_client_as_lib};
#[cfg(feature = "wasm-builder")]
use std::path::PathBuf;
#[cfg(feature = "wasm-builder")]
pub fn build_wasm() -> Option<(PathBuf, PathBuf)> {
    if let Err(err) = sails_build_support::ensure_canonical_env() {
        panic!("failed to generate canonical document: {err}");
    }
    if std::env::var_os("__GEAR_WASM_BUILDER_NO_BUILD").is_some() {
        write_stub_wasm_binary()
            .expect("failed to write wasm binary stub while canonical generation is disabled");
        return None;
    }
    gwasm_builder::build()
}
pub use hex;
pub use prelude::*;
#[cfg(feature = "idl-gen")]
#[cfg(not(target_arch = "wasm32"))]
pub use sails_idl_gen::{generate_idl, generate_idl_to_file};
pub use sails_idl_meta::{self as meta};
#[cfg(feature = "std")]
pub use sails_interface_id as interface_id;
#[cfg(not(feature = "std"))]
pub mod interface_id {}
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
pub use sails_program_registry as program_registry;
#[cfg(any(not(feature = "std"), target_arch = "wasm32"))]
pub mod program_registry {
    #[macro_export]
    macro_rules! submit_program_registration {
        ($registration:expr) => {
            #[cfg(not(target_arch = "wasm32"))]
            compile_error!(
                "`sails-rs` must be linked with the `std` feature to register programs on \
                 the host. Update your dependency to `sails-rs = { features = [\"std\"] }`."
            );
        };
    }
}
pub use spin;

#[cfg(feature = "wasm-builder")]
fn write_stub_wasm_binary() -> std::io::Result<()> {
    use std::io::Write;

    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let path = PathBuf::from(out_dir).join("wasm_binary.rs");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(path)?;
        file.write_all(
            br#"#[allow(unused)]
pub const WASM_BINARY: &[u8] = &[];
#[allow(unused)]
pub const WASM_BINARY_OPT: &[u8] = &[];
"#,
        )?;
    }

    Ok(())
}

#[cfg(feature = "client-builder")]
mod builder;
pub mod client;
pub mod errors;
#[cfg(feature = "gclient")]
#[cfg(not(target_arch = "wasm32"))]
pub use gclient;
#[cfg(feature = "gstd")]
pub mod gstd;
#[cfg(feature = "gtest")]
#[cfg(not(target_arch = "wasm32"))]
pub use gtest;
#[cfg(feature = "mockall")]
#[cfg(not(target_arch = "wasm32"))]
pub use mockall;
pub mod prelude;
#[cfg(feature = "ethexe")]
pub mod solidity;
mod types;
mod utils;
