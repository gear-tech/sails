#![no_std]

#[cfg(target_arch = "wasm32")]
pub use demo_app::wasm::*;
