#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(target_arch = "wasm32")]
use talc::{source::Claim, *};

pub mod ast;
mod grammar;
mod lexer;

pub mod ffi {
    pub mod ast;
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static TALC: TalcLock<spinning_top::RawSpinlock, Claim> = TalcLock::new(unsafe {
    static mut INITIAL_HEAP: [u8; min_first_heap_size::<DefaultBinning>() + 100000] =
        [0; min_first_heap_size::<DefaultBinning>() + 100000];

    Claim::array(&raw mut INITIAL_HEAP)
});

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}
