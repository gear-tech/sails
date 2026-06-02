#![no_std]

#[macro_use]
extern crate alloc;
extern crate galloc;

pub mod ast;
mod grammar;
mod lexer;

pub mod ffi {
    pub mod ast;
}

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}
