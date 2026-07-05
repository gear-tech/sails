#![cfg_attr(target_arch = "wasm32", no_std)]

#[macro_use]
extern crate alloc;

mod error;
#[cfg(feature = "ffi")]
pub mod ffi;
mod generator;
mod sol_conversion;

pub use error::*;
pub use generator::{
    LICENSE_IDENTIFIER, SOLIDITY_VERSION, SolidityFile, generate_solidity_contract,
};
pub use sol_conversion::ConversionError;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static TALC: talc::wasm::WasmDynamicTalc = talc::wasm::new_wasm_dynamic_allocator();

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}
