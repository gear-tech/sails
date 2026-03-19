extern crate alloc;

use alloc::string::String;
use sails_type_registry::{
    Registry,
    ty::{Primitive, TypeDef, TypeDefinitionKind},
};

macro_rules! assert_primitive {
    ($registry:expr, $t:ty, $expected_primitive:ident) => {
        let id = $registry.register_type::<$t>();
        let ty = $registry.get_type(id).expect("Type should be in registry");
        assert_eq!(ty.def, TypeDef::Primitive(Primitive::$expected_primitive));
        assert!(
            ty.module_path.is_empty(),
            "Primitives should not have a path"
        );
        assert!(
            ty.annotations.is_empty(),
            "Primitives should not have annotations"
        );
    };
}

#[test]
fn test_standard_primitives() {
    let mut registry = Registry::new();

    // Booleans and characters
    assert_primitive!(registry, bool, Bool);
    assert_primitive!(registry, char, Char);

    // Strings
    assert_primitive!(registry, str, Str);
    assert_primitive!(registry, String, Str);

    // Unsigned Integers
    assert_primitive!(registry, u8, U8);
    assert_primitive!(registry, u16, U16);
    assert_primitive!(registry, u32, U32);
    assert_primitive!(registry, u64, U64);
    assert_primitive!(registry, u128, U128);

    // Signed Integers
    assert_primitive!(registry, i8, I8);
    assert_primitive!(registry, i16, I16);
    assert_primitive!(registry, i32, I32);
    assert_primitive!(registry, i64, I64);
    assert_primitive!(registry, i128, I128);
}

#[test]
fn test_nonzero_is_composite() {
    use core::num::NonZeroU8;
    let mut registry = Registry::new();

    let id = registry.register_type::<NonZeroU8>();
    let ty = registry.get_type(id).expect("Type should be in registry");

    // NonZero is now Composite, not Primitive
    if let TypeDef::Definition(def) = &ty.def
        && let TypeDefinitionKind::Composite(c) = &def.kind
    {
        assert_eq!(c.fields.len(), 1);
        assert!(registry.is_type::<u8>(c.fields[0].ty.id()));
    } else {
        panic!("NonZeroU8 should be composite, got {:?}", ty.def);
    }
    assert_eq!(ty.name, "NonZeroU8");
}
