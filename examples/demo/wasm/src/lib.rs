#![no_std]

// Re-export functions required for buildng WASM app
#[cfg(target_arch = "wasm32")]
pub use demo_app::wasm::*;
