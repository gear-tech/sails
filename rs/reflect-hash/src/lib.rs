//! Structural compile-time hashing for Sails types.
//!
//! This crate provides the `ReflectHash` trait which computes a deterministic,
//! name-independent structural hash for types at compile time using const evaluation.
//! This enables unique interface IDs for services based purely on their structure.

// todo [sab] check the crate

#![no_std]

extern crate alloc;

use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

/// Core trait for computing structural compile-time hashes.
///
/// Types implementing this trait can be hashed at compile time to produce
/// a unique 32-byte identifier based solely on their structure, not their names.
pub trait ReflectHash {
    /// The 256-bit structural hash of this type, computed at compile time.
    const HASH: [u8; 32];
}

// ============================================================================
// Primitive Integer Types
// ============================================================================

macro_rules! impl_reflect_hash_for_primitives {
    ($($t:ty => $discriminant:literal),* $(,)?) => {
        $(
            impl ReflectHash for $t {
                const HASH: [u8; 32] = keccak_const::Keccak256::new()
                    .update($discriminant)
                    .finalize();
            }
        )*
    };
}

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
}

// ============================================================================
// NonZero Integer Types
// ============================================================================

impl ReflectHash for NonZeroU8 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroU8")
        .finalize();
}

impl ReflectHash for NonZeroU16 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroU16")
        .finalize();
}

impl ReflectHash for NonZeroU32 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroU32")
        .finalize();
}

impl ReflectHash for NonZeroU64 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroU64")
        .finalize();
}

impl ReflectHash for NonZeroU128 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroU128")
        .finalize();
}

impl ReflectHash for NonZeroI8 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroI8")
        .finalize();
}

impl ReflectHash for NonZeroI16 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroI16")
        .finalize();
}

impl ReflectHash for NonZeroI32 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroI32")
        .finalize();
}

impl ReflectHash for NonZeroI64 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroI64")
        .finalize();
}

impl ReflectHash for NonZeroI128 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroI128")
        .finalize();
}

// ============================================================================
// String Types & Slices
// ============================================================================

// Note: str has the hash for "String" since they represent
// the same logical type in a structural interface
impl ReflectHash for str {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"String")
        .finalize();
}

// [T] (slice) has same structure as Vec<T> 
impl<T: ReflectHash> ReflectHash for [T] {
    const HASH: [u8; 32] = {
        keccak_const::Keccak256::new()
            .update(b"Vec")
            .update(&T::HASH)
            .finalize()
    };
}

// ============================================================================
// Reference Types
// ============================================================================

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

// ============================================================================
// Unit Type
// ============================================================================

impl ReflectHash for () {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"()")
        .finalize();
}

// ============================================================================
// Option<T>
// ============================================================================

impl<T: ReflectHash> ReflectHash for Option<T> {
    const HASH: [u8; 32] = {
        keccak_const::Keccak256::new()
            .update(b"Option")
            .update(&T::HASH)
            .finalize()
    };
}

// ============================================================================
// Result<T, E>
// ============================================================================

impl<T: ReflectHash, E: ReflectHash> ReflectHash for Result<T, E> {
    const HASH: [u8; 32] = {
        keccak_const::Keccak256::new()
            .update(b"Result")
            .update(&T::HASH)
            .update(&E::HASH)
            .finalize()
    };
}

// ============================================================================
// Tuples (up to 12 elements)
// ============================================================================

macro_rules! impl_reflect_hash_for_tuples {
    () => {};
    ($first:ident $(, $rest:ident)*) => {
        impl<$first: ReflectHash, $($rest: ReflectHash),*> ReflectHash for ($first, $($rest),*) {
            const HASH: [u8; 32] = {
                keccak_const::Keccak256::new()
                    .update(b"(")
                    .update(&$first::HASH)
                    $(
                        .update(&$rest::HASH)
                    )*
                    .update(b")")
                    .finalize()
            };
        }
        impl_reflect_hash_for_tuples!($($rest),*);
    };
}

impl_reflect_hash_for_tuples!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

// ============================================================================
// Arrays (fixed size up to 32)
// ============================================================================

macro_rules! impl_reflect_hash_for_arrays {
    ($($n:expr),* $(,)?) => {
        $(
            impl<T: ReflectHash> ReflectHash for [T; $n] {
                const HASH: [u8; 32] = {
                    keccak_const::Keccak256::new()
                        .update(b"[")
                        .update(&T::HASH)
                        .update(b";")
                        .update(&[$n as u8]) // Size encoded as bytes
                        .update(b"]")
                        .finalize()
                };
            }
        )*
    };
}

impl_reflect_hash_for_arrays!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32,
);

// ============================================================================
// Gear Primitive Types
// ============================================================================

impl ReflectHash for gprimitives::ActorId {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"ActorId")
        .finalize();
}

impl ReflectHash for gprimitives::MessageId {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"MessageId")
        .finalize();
}

impl ReflectHash for gprimitives::CodeId {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"CodeId")
        .finalize();
}

impl ReflectHash for gprimitives::H256 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"H256")
        .finalize();
}

impl ReflectHash for gprimitives::H160 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"H160")
        .finalize();
}

impl ReflectHash for gprimitives::U256 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"U256")
        .finalize();
}

impl ReflectHash for gprimitives::NonZeroU256 {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"NonZeroU256")
        .finalize();
}

// ============================================================================
// Collection Types (Temporary Mock Hashes)
// ============================================================================

// TODO: Implement proper structural hashing for Vec<T>
// For now, using a mock implementation
impl<T: ReflectHash> ReflectHash for alloc::vec::Vec<T> {
    const HASH: [u8; 32] = {
        keccak_const::Keccak256::new()
            .update(b"Vec")
            .update(&T::HASH)
            .finalize()
    };
}

// TODO: Implement proper structural hashing for BTreeMap<K, V>
// For now, using a mock implementation with a comment
// Note: This is a placeholder - proper implementation would require
// const-compatible BTreeMap operations
// impl<K: ReflectHash, V: ReflectHash> ReflectHash for alloc::collections::BTreeMap<K, V> {
//     const HASH: [u8; 32] = {
//         keccak_const::Keccak256::new()
//             .update(b"BTreeMap")
//             .update(&K::HASH)
//             .update(&V::HASH)
//             .finalize()
//     };
// }

// Note: String type (owned) has same structural hash as str
impl ReflectHash for alloc::string::String {
    const HASH: [u8; 32] = <str as ReflectHash>::HASH;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives_have_different_hashes() {
        assert_ne!(u8::HASH, u16::HASH);
        assert_ne!(u32::HASH, i32::HASH);
        assert_ne!(bool::HASH, u8::HASH);
    }

    #[test]
    fn test_option_hashes() {
        assert_ne!(Option::<u8>::HASH, Option::<u16>::HASH);
        assert_ne!(Option::<u32>::HASH, u32::HASH);
    }

    #[test]
    fn test_result_hashes() {
        assert_ne!(Result::<u8, u16>::HASH, Result::<u16, u8>::HASH);
        assert_ne!(Result::<u32, bool>::HASH, u32::HASH);
    }

    #[test]
    fn test_tuple_hashes() {
        type Tuple1 = (u8, u16);
        type Tuple2 = (u16, u8);
        type Tuple3 = (u32,);
        type Tuple4 = (u8, u16, bool);
        
        assert_ne!(Tuple1::HASH, Tuple2::HASH);
        assert_ne!(Tuple3::HASH, u32::HASH);
        assert_ne!(Tuple1::HASH, Tuple4::HASH);
    }

    #[test]
    fn test_array_hashes() {
        type Array1 = [u8; 4];
        type Array2 = [u8; 8];
        type Array3 = [u16; 4];
        
        assert_ne!(Array1::HASH, Array2::HASH);
        assert_ne!(Array1::HASH, Array3::HASH);
    }

    #[test]
    fn test_unit_hash() {
        // Unit type should have a deterministic hash
        let _ = <()>::HASH;
    }

    #[test]
    fn test_nonzero_hashes() {
        assert_ne!(core::num::NonZeroU8::HASH, u8::HASH);
        assert_ne!(core::num::NonZeroU32::HASH, core::num::NonZeroU64::HASH);
    }

    #[test]
    fn test_gear_types_hashes() {
        use gprimitives::{ActorId, CodeId, H160, H256, MessageId, NonZeroU256, U256};
        
        // All Gear types should have different hashes
        assert_ne!(ActorId::HASH, MessageId::HASH);
        assert_ne!(ActorId::HASH, CodeId::HASH);
        assert_ne!(H256::HASH, H160::HASH);
        assert_ne!(U256::HASH, NonZeroU256::HASH);
        
        // Gear types should differ from primitives
        assert_ne!(ActorId::HASH, <[u8; 32]>::HASH);
        assert_ne!(H256::HASH, <[u8; 32]>::HASH);
    }

    #[test]
    fn test_string_types() {
        // String and &str should have the same structural hash
        assert_eq!(alloc::string::String::HASH, <&str>::HASH);
    }

    #[test]
    fn test_vec_hash() {
        use alloc::vec::Vec;
        
        assert_ne!(Vec::<u8>::HASH, Vec::<u16>::HASH);
        assert_ne!(Vec::<u32>::HASH, u32::HASH);
    }
}
