#![no_std]

#[cfg(target_arch = "wasm32")]
pub use this_that_svc_app::wasm::*;
