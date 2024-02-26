#![no_std]

// Temporary re-export for using in rmrk-resource tests.
// Later the tests should be re-written using generated client
pub use rmrk_catalog_app::services;
#[cfg(target_arch = "wasm32")]
pub use rmrk_catalog_app::wasm::*;
#[cfg(target_arch = "wasm32")]
pub use rmrk_catalog_app::wasm_main::*;
