#![no_std]

#[cfg(not(target_arch = "wasm32"))]
pub mod client;

// Re-export functions required for buildng WASM app
#[cfg(target_arch = "wasm32")]
pub use rmrk_resource_app::wasm::*;
