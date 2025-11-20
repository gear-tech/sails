//! Tests for #[derive(ReflectHash)]

use sails_reflect_hash::ReflectHash;
use keccak_const::Keccak256;

// ============================================================================
// Struct Tests
// ============================================================================

#[derive(ReflectHash)]
struct UnitStruct;

#[derive(ReflectHash)]
struct TupleStruct(u32, u64);

#[derive(ReflectHash)]
struct NamedStruct {
    field_a: u32,
    field_b: u64,
}

#[test]
fn test_unit_struct_derive() {
    // Manual computation: keccak256(b"UnitStruct")
    let manual_hash = Keccak256::new()
        .update(b"UnitStruct")
        .finalize();
    
    assert_eq!(UnitStruct::HASH, manual_hash);
}

#[test]
fn test_tuple_struct_derive() {
    // Manual computation: keccak256(b"TupleStruct" || u32::HASH || u64::HASH)
    let manual_hash = Keccak256::new()
        .update(b"TupleStruct")
        .update(&u32::HASH)
        .update(&u64::HASH)
        .finalize();
    
    assert_eq!(TupleStruct::HASH, manual_hash);
}

#[test]
fn test_named_struct_derive() {
    // Manual computation: keccak256(b"NamedStruct" || u32::HASH || u64::HASH)
    // Field names are NOT included
    let manual_hash = Keccak256::new()
        .update(b"NamedStruct")
        .update(&u32::HASH)
        .update(&u64::HASH)
        .finalize();
    
    assert_eq!(NamedStruct::HASH, manual_hash);
}

// ============================================================================
// Enum Tests
// ============================================================================

#[derive(ReflectHash)]
enum SimpleEnum {
    Variant1,
    Variant2,
    Variant3,
}

#[derive(ReflectHash)]
enum MixedEnum {
    Unit,
    Tuple(u32, u64),
    Named { x: u32, y: u64 },
}

#[test]
fn test_simple_enum_derive() {
    // Variant hashes
    let v1 = Keccak256::new().update(b"Variant1").finalize();
    let v2 = Keccak256::new().update(b"Variant2").finalize();
    let v3 = Keccak256::new().update(b"Variant3").finalize();
    
    // Final enum hash
    let manual_hash = Keccak256::new()
        .update(&v1)
        .update(&v2)
        .update(&v3)
        .finalize();
    
    assert_eq!(SimpleEnum::HASH, manual_hash);
}

#[test]
fn test_mixed_enum_derive() {
    // Variant 1: Unit
    let v1 = Keccak256::new()
        .update(b"Unit")
        .finalize();
    
    // Variant 2: Tuple(u32, u64)
    let v2 = Keccak256::new()
        .update(b"Tuple")
        .update(&u32::HASH)
        .update(&u64::HASH)
        .finalize();
    
    // Variant 3: Named { x: u32, y: u64 }
    // Field names are NOT included - only types
    let v3 = Keccak256::new()
        .update(b"Named")
        .update(&u32::HASH)
        .update(&u64::HASH)
        .finalize();
    
    // Final enum hash
    let manual_hash = Keccak256::new()
        .update(&v1)
        .update(&v2)
        .update(&v3)
        .finalize();
    
    assert_eq!(MixedEnum::HASH, manual_hash);
}

// ============================================================================
// Complex Event-like Enum
// ============================================================================

#[cfg(feature = "gprimitives")]
#[derive(ReflectHash)]
enum TransferEvent {
    Transferred { from: gprimitives::ActorId, to: gprimitives::ActorId, amount: u128 },
    Approved(gprimitives::ActorId, u128),
    Paused,
}

#[cfg(feature = "gprimitives")]
#[test]
fn test_transfer_event_derive() {
    use gprimitives::ActorId;
    
    // Variant 1: Transferred { from, to, amount }
    // Field names are NOT included - only types
    let v1 = Keccak256::new()
        .update(b"Transferred")
        .update(&ActorId::HASH)
        .update(&ActorId::HASH)
        .update(&u128::HASH)
        .finalize();
    
    // Variant 2: Approved(ActorId, u128)
    let v2 = Keccak256::new()
        .update(b"Approved")
        .update(&ActorId::HASH)
        .update(&u128::HASH)
        .finalize();
    
    // Variant 3: Paused
    let v3 = Keccak256::new()
        .update(b"Paused")
        .finalize();
    
    // Final hash
    let manual_hash = Keccak256::new()
        .update(&v1)
        .update(&v2)
        .update(&v3)
        .finalize();
    
    assert_eq!(TransferEvent::HASH, manual_hash);
}

// ============================================================================
// Field Names Don't Matter (same struct, different field names via enums)
// ============================================================================

#[derive(ReflectHash)]
enum SameTypesEnum1 {
    Variant { a: u32, b: u64 }
}

#[derive(ReflectHash)]
enum SameTypesEnum2 {
    Variant { x: u32, y: u64 }
}

#[test]
fn test_field_names_ignored() {
    // Same variant name, same types, different field names = same hash
    assert_eq!(SameTypesEnum1::HASH, SameTypesEnum2::HASH);
}

// ============================================================================
// Field Order Matters
// ============================================================================

#[derive(ReflectHash)]
struct Order1 { a: u32, b: u64 }

#[derive(ReflectHash)]
struct Order2 { b: u64, a: u32 }

#[test]
fn test_field_order_matters() {
    // Different field order should produce different hashes
    // Order1: u32, u64 vs Order2: u64, u32
    assert_ne!(Order1::HASH, Order2::HASH);
}

// ============================================================================
// Type Variations
// ============================================================================

#[derive(ReflectHash)]
enum TypeVariations {
    IntType(u32),
    StringType(u64),
}

#[test]
fn test_type_variations() {
    // Even with same structure, different types should differ
    let v1 = Keccak256::new()
        .update(b"IntType")
        .update(&u32::HASH)
        .finalize();
    
    let v2 = Keccak256::new()
        .update(b"StringType")
        .update(&u64::HASH)
        .finalize();
    
    assert_ne!(v1, v2);
}

// ============================================================================
// Nested Types
// ============================================================================

#[derive(ReflectHash)]
struct Inner {
    value: u32,
}

#[derive(ReflectHash)]
struct Outer {
    inner: Inner,
    extra: u64,
}

#[test]
fn test_nested_types() {
    // Inner hash - field names not included
    let inner_hash = Keccak256::new()
        .update(b"Inner")
        .update(&u32::HASH)
        .finalize();
    
    assert_eq!(Inner::HASH, inner_hash);
    
    // Outer hash uses Inner::HASH - field names not included
    let outer_hash = Keccak256::new()
        .update(b"Outer")
        .update(&Inner::HASH)
        .update(&u64::HASH)
        .finalize();
    
    assert_eq!(Outer::HASH, outer_hash);
}

// ============================================================================
// Empty Enum (edge case)
// ============================================================================

#[derive(ReflectHash)]
enum EmptyEnum {}

#[test]
fn test_empty_enum() {
    // Empty enum should have a hash (even though it's uninhabited)
    let manual_hash = Keccak256::new().finalize();
    assert_eq!(EmptyEnum::HASH, manual_hash);
}

// ============================================================================
// Single Variant Enum
// ============================================================================

#[derive(ReflectHash)]
enum SingleVariant {
    Only(u32),
}

#[test]
fn test_single_variant_enum() {
    let v1 = Keccak256::new()
        .update(b"Only")
        .update(&u32::HASH)
        .finalize();
    
    let manual_hash = Keccak256::new()
        .update(&v1)
        .finalize();
    
    assert_eq!(SingleVariant::HASH, manual_hash);
}
