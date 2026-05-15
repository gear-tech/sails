//! Tests for nested types

use keccak_const::Keccak256;
use sails_reflect_hash::ReflectHash;

#[derive(ReflectHash)]
#[allow(dead_code)]
struct Inner {
    value: u32,
}

#[derive(ReflectHash)]
#[allow(dead_code)]
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
