use sails_idl_ast::{PrimitiveType, TypeDef};
use sails_type_registry::{Registry, TypeDecl};

macro_rules! assert_primitive {
    ($registry:expr, $t:ty, $expected_primitive:ident) => {
        let id = $registry.register_type::<$t>();
        let decl = $registry
            .get_type_decl(id)
            .expect(concat!(stringify!($t), " should have a type declaration"));
        assert_eq!(
            *decl,
            TypeDecl::Primitive(PrimitiveType::$expected_primitive)
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
    assert_primitive!(registry, str, String);
    assert_primitive!(registry, String, String);

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
    use sails_idl_ast::StructDef;
    let mut registry = Registry::new();

    let id = registry.register_type::<NonZeroU8>();
    let _decl = registry.get_type_decl(id).expect(concat!(
        stringify!(NonZeroU8),
        " should have a type declaration"
    ));

    // NonZero is a Named type with a struct definition
    let type_def = registry.get_type(id).expect(concat!(
        stringify!(NonZeroU8),
        " should have a type definition"
    ));
    match &type_def.def {
        TypeDef::Struct(StructDef { fields }) => {
            assert_eq!(fields.len(), 1);
            match &fields[0].type_decl {
                TypeDecl::Primitive(PrimitiveType::U8) => {}
                _other => panic!(concat!(stringify!(NonZeroU8), " should have U8 field"),),
            }
        }
        _ => panic!(concat!(stringify!(NonZeroU8), " should be a struct"),),
    }
}
