//! Tests for field semantics (names, order, types)

use sails_reflect_hash::ReflectHash;

// ============================================================================
// Field Names Don't Matter
// ============================================================================

#[derive(ReflectHash)]
#[allow(dead_code)]
enum SameTypesEnum1 {
    Variant { a: u32, b: u64 },
}

#[derive(ReflectHash)]
#[allow(dead_code)]
enum SameTypesEnum2 {
    Variant { x: u32, y: u64 },
}

#[test]
fn test_field_names_ignored() {
    assert_eq!(SameTypesEnum1::HASH, SameTypesEnum2::HASH);
}

// ============================================================================
// Field Order Matters
// ============================================================================

#[derive(ReflectHash)]
#[allow(dead_code)]
struct Order1 {
    a: u32,
    b: u64,
}

#[derive(ReflectHash)]
#[allow(dead_code)]
struct Order2 {
    b: u64,
    a: u32,
}

#[test]
fn test_field_order_matters() {
    assert_ne!(Order1::HASH, Order2::HASH);
}
