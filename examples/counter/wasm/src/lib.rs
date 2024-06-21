#![no_std]

#[cfg(target_arch = "wasm32")]
pub use counter_app::wasm::*;
