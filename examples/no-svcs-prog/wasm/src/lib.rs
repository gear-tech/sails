#![no_std]

#[cfg(target_arch = "wasm32")]
pub use no_svcs_prog_app::wasm::*;
