#![cfg_attr(target_arch = "wasm32", no_std)]

#[macro_use]
extern crate alloc;

pub mod ast;
mod grammar;
mod lexer;

pub mod ffi {
    pub mod ast;
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static TALC: talc::wasm::WasmDynamicTalc = talc::wasm::new_wasm_dynamic_allocator();

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}
