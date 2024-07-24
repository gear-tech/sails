#![no_std]

#[cfg(not(target_arch = "wasm32"))]
pub mod client;

#[cfg(target_arch = "wasm32")]
pub use no_svcs_prog_app::wasm::*;
