#![no_std]

#[cfg(target_arch = "wasm32")]
pub use rmrk_resource_app::wasm::*;
#[cfg(target_arch = "wasm32")]
pub use rmrk_resource_app::wasm_main::*;
