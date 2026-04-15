#![cfg(not(feature = "ethexe"))]

use sails_rs::{Decode, Encode};

mod sails_type_roundtrip;

use sails_type_roundtrip::{LegacyType, MyEnum, MyType};

#[test]
fn struct_round_trips() {
    let value = MyType {
        a: 42,
        b: "hello".to_string(),
    };
    let bytes = value.encode();
    let decoded = MyType::decode(&mut &bytes[..]).unwrap();
    assert_eq!(value, decoded);
}

#[test]
fn enum_round_trips() {
    let value = MyEnum::Named { x: 7 };
    let bytes = value.encode();
    let decoded = MyEnum::decode(&mut &bytes[..]).unwrap();
    assert_eq!(value, decoded);
}

#[test]
fn legacy_type_round_trips() {
    let value = LegacyType { a: 99 };
    let bytes = value.encode();
    let decoded = LegacyType::decode(&mut &bytes[..]).unwrap();
    assert_eq!(value, decoded);
}
