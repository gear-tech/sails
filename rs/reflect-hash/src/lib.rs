// This file is part of Gear.

// Copyright (C) 2025 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Structural compile-time hashing for Sails types.
//!
//! This crate provides the `ReflectHash` trait which computes a deterministic,
//! name-independent structural hash for types at compile time using const evaluation.
//! This enables unique interface IDs for services based purely on their structure.
//!
//! # Deriving ReflectHash
//!
//! The easiest way to implement `ReflectHash` is to derive it:
//!
//! ```ignore
//! use sails_reflect_hash::ReflectHash;
//!
//! #[derive(ReflectHash)]
//! struct Transfer {
//!     from: ActorId,
//!     to: ActorId,
//!     amount: u128,
//! }
//!
//! #[derive(ReflectHash)]
//! enum Event {
//!     Transferred { from: ActorId, to: ActorId },
//!     Approved(ActorId, u128),
//!     Paused,
//! }
//! ```

#![no_std]

extern crate alloc;

// Re-export the derive macro
pub use sails_reflect_hash_derive::ReflectHash;

// Re-export dependencies needed by the derive macro
#[doc(hidden)]
pub use keccak_const;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroU8, NonZeroU16, NonZeroU32,
    NonZeroU64, NonZeroU128,
};
use gprimitives::{ActorId, CodeId, H160, H256, MessageId, NonZeroU256, U256};
use keccak_const::Keccak256;

/// Core trait for computing structural compile-time hashes.
///
/// Types implementing this trait can be hashed at compile time to produce
/// a unique 32-byte identifier based solely on their structure, not their names.
///
/// # Deriving
///
/// Most types should use `#[derive(ReflectHash)]` which automatically generates
/// the correct implementation based on the type's structure.
pub trait ReflectHash {
    /// The 256-bit structural hash of this type, computed at compile time.
    const HASH: [u8; 32];
}

macro_rules! impl_reflect_hash_for_primitives {
    ($($t:ty => $discriminant:literal),* $(,)?) => {
        $(
            impl ReflectHash for $t {
                const HASH: [u8; 32] = Keccak256::new()
                    .update($discriminant)
                    .finalize();
            }
        )*
    };
}

// Note: str has the hash for "String" since they represent
// the same logical type in a structural interface and in IDL.
impl_reflect_hash_for_primitives! {
    u8 => b"u8",
    u16 => b"u16",
    u32 => b"u32",
    u64 => b"u64",
    u128 => b"u128",
    i8 => b"i8",
    i16 => b"i16",
    i32 => b"i32",
    i64 => b"i64",
    i128 => b"i128",
    bool => b"bool",
    char => b"char",
    str => b"String",
    String => b"String",
}

impl ReflectHash for NonZeroU8 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroU8")
        .update(&<u8 as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for NonZeroU16 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroU16")
        .update(&<u16 as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for NonZeroU32 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroU32")
        .update(&<u32 as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for NonZeroU64 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroU64")
        .update(&<u64 as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for NonZeroU128 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroU128")
        .update(&<u128 as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for NonZeroI8 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroI8")
        .update(&<i8 as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for NonZeroI16 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroI16")
        .update(&<i16 as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for NonZeroI32 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroI32")
        .update(&<i32 as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for NonZeroI64 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroI64")
        .update(&<i64 as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for NonZeroI128 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroI128")
        .update(&<i128 as ReflectHash>::HASH)
        .finalize();
}

// [T] (slice) has same data structure as Vec<T>
impl<T: ReflectHash> ReflectHash for [T] {
    const HASH: [u8; 32] = { Keccak256::new().update(b"Vec").update(&T::HASH).finalize() };
}

// Immutable references have the same hash as the referent
// (structural equivalence: &T ≡ T in interface terms)
impl<T: ReflectHash + ?Sized> ReflectHash for &T {
    const HASH: [u8; 32] = T::HASH;
}

// Mutable references have the same hash as the referent
// (structural equivalence: &mut T ≡ T in interface terms)
impl<T: ReflectHash + ?Sized> ReflectHash for &mut T {
    const HASH: [u8; 32] = T::HASH;
}

impl ReflectHash for () {
    const HASH: [u8; 32] = Keccak256::new().update(b"()").finalize();
}

impl<T: ReflectHash> ReflectHash for Option<T> {
    const HASH: [u8; 32] = {
        Keccak256::new()
            .update(b"Option")
            .update(&T::HASH)
            .finalize()
    };
}

impl<T: ReflectHash, E: ReflectHash> ReflectHash for Result<T, E> {
    const HASH: [u8; 32] = {
        Keccak256::new()
            .update(b"Result")
            .update(&T::HASH)
            .update(&E::HASH)
            .finalize()
    };
}

macro_rules! impl_reflect_hash_for_tuples {
    () => {};
    ($first:ident $(, $rest:ident)*) => {
        impl<$first: ReflectHash, $($rest: ReflectHash),*> ReflectHash for ($first, $($rest),*) {
            const HASH: [u8; 32] = {
                Keccak256::new()
                    .update(&$first::HASH)
                    $(
                        .update(&$rest::HASH)
                    )*
                    .finalize()
            };
        }
        impl_reflect_hash_for_tuples!($($rest),*);
    };
}

// Implement ReflectHash for tuples up to 12 elements
impl_reflect_hash_for_tuples!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

macro_rules! impl_reflect_hash_for_bytes_arrays {
    ($($n:expr),* $(,)?) => {
        $(
            impl<T: ReflectHash> ReflectHash for [T; $n] {
                const HASH: [u8; 32] = {
                    let n_str = stringify!($n);
                    Keccak256::new()
                        .update(&T::HASH)
                        .update(n_str.as_bytes())
                        .finalize()
                };
            }
        )*
    };
}

// Implement ReflectHash for arrays up to size 32
impl_reflect_hash_for_bytes_arrays!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32,
);

impl ReflectHash for ActorId {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"ActorId")
        .update(&<[u8; 32] as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for MessageId {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"MessageId")
        .update(&<[u8; 32] as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for CodeId {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"CodeId")
        .update(&<[u8; 32] as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for H256 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"H256")
        .update(&<[u8; 32] as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for H160 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"H160")
        .update(&<[u8; 20] as ReflectHash>::HASH)
        .finalize();
}

impl ReflectHash for U256 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"U256")
        .update(
            &Keccak256::new()
                .update(&<u64 as ReflectHash>::HASH)
                .update(b"4")
                .finalize(),
        )
        .finalize();
}

impl ReflectHash for NonZeroU256 {
    const HASH: [u8; 32] = Keccak256::new()
        .update(b"NonZeroU256")
        .update(&U256::HASH)
        .finalize();
}

impl<T: ReflectHash> ReflectHash for Vec<T> {
    const HASH: [u8; 32] = { Keccak256::new().update(b"Vec").update(&T::HASH).finalize() };
}

impl<K: ReflectHash, V: ReflectHash> ReflectHash for BTreeMap<K, V> {
    const HASH: [u8; 32] = {
        Keccak256::new()
            .update(b"BTreeMap")
            .update(&K::HASH)
            .update(&V::HASH)
            .finalize()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_hash_types() {
        assert_eq!(str::HASH, String::HASH);
        assert_eq!(<[u8] as ReflectHash>::HASH, Vec::<u8>::HASH);
    }

    // Test it builds and works
    #[test]
    fn crate_paths() {
        use crate as reflect_hash_crate;

        #[derive(ReflectHash)]
        #[reflect_hash(crate = reflect_hash_crate)]
        #[allow(dead_code)]
        struct TestStruct(String);
    }
}
