#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![no_std]

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
extern crate std;

#[cfg(feature = "client-builder")]
pub use builder::{ClientBuilder, ClientGenerator, IdlPath, build_client, build_client_as_lib};
#[cfg(feature = "wasm-builder")]
pub use gwasm_builder::build as build_wasm;
pub use hex;
pub use prelude::*;
#[cfg(all(feature = "idl-gen", not(target_arch = "wasm32")))]
pub use sails_idl_gen::generate_idl;
#[cfg(all(feature = "idl-gen", feature = "std", not(target_arch = "wasm32")))]
pub use sails_idl_gen::generate_idl_to_file;
pub use sails_idl_meta::{self as meta};
pub use spin;

#[cfg(feature = "client-builder")]
mod builder;
pub mod client;
pub mod errors;
#[cfg(all(feature = "gclient", not(target_arch = "wasm32")))]
pub use gclient;
#[cfg(feature = "gstd")]
pub mod gstd;
#[cfg(all(feature = "gtest", not(target_arch = "wasm32")))]
pub use gtest;
#[cfg(all(feature = "mockall", not(target_arch = "wasm32")))]
pub use mockall;
pub mod prelude;
#[cfg(feature = "ethexe")]
pub mod solidity;
mod types;
mod utils;
