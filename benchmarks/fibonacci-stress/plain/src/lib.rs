#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
#[cfg(not(feature = "std"))]
mod wasm;

#[cfg(feature = "std")]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

use parity_scale_codec::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub enum Action {
    StressFiboOptimized(u32),
    StressFibo(u32),
    StressBytesOptimized(u32),
    StressBytes(u32),
}

impl Action {
    pub fn n(&self) -> u32 {
        match self {
            Action::StressFiboOptimized(n) => *n,
            Action::StressFibo(n) => *n,
            Action::StressBytesOptimized(n) => *n,
            Action::StressBytes(n) => *n,
        }
    }

    pub fn is_fibo(&self) -> bool {
        matches!(self, Action::StressFiboOptimized(_) | Action::StressFibo(_))
    }
}
