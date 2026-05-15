//! Tests for #[derive(ReflectHash)] on structs

use keccak_const::Keccak256;
use sails_reflect_hash::ReflectHash;

#[derive(ReflectHash)]
#[allow(dead_code)]
struct UnitStruct;

#[derive(ReflectHash)]
#[allow(dead_code)]
struct TupleStruct(u32, u64);

#[derive(ReflectHash)]
#[allow(dead_code)]
struct NamedStruct {
    field_a: u32,
    field_b: u64,
}

#[test]
fn test_unit_struct_derive() {
    // Manual computation: keccak256(b"UnitStruct")
    let manual_hash = Keccak256::new().update(b"UnitStruct").finalize();

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
