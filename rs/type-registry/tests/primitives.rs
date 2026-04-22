use alloc::string::String;
use sails_idl_ast::{PrimitiveType, TypeDecl};
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeInfo};

macro_rules! assert_primitive {
    ($registry:expr, $ty:ty, $expected:ident) => {
        assert_eq!(
            <$ty as TypeInfo>::type_decl(&mut $registry),
            TypeDecl::Primitive(PrimitiveType::$expected),
        );
    };
}

#[test]
fn standard_primitives_lower_to_primitive_type_decl() {
    let mut registry = Registry::new();

    assert_primitive!(registry, bool, Bool);
    assert_primitive!(registry, char, Char);

    assert_primitive!(registry, str, String);
    assert_primitive!(registry, String, String);

    assert_primitive!(registry, u8, U8);
    assert_primitive!(registry, u16, U16);
    assert_primitive!(registry, u32, U32);
    assert_primitive!(registry, u64, U64);
    assert_primitive!(registry, u128, U128);

    assert_primitive!(registry, i8, I8);
    assert_primitive!(registry, i16, I16);
    assert_primitive!(registry, i32, I32);
    assert_primitive!(registry, i64, I64);
    assert_primitive!(registry, i128, I128);

    assert_primitive!(registry, (), Void);

    assert!(
        registry.is_empty(),
        "primitives must not allocate registry slots"
    );
}

#[test]
fn nonzero_integer_registers_as_nominal_wrapper() {
    use core::num::NonZeroU8;
    use sails_idl_ast::TypeDef;

    let mut registry = Registry::new();

    let type_ref = registry
        .register_type::<NonZeroU8>()
        .expect("NonZero is nominal");
    let ty = registry.get_type(type_ref).unwrap();

    assert_eq!(ty.name, "NonZeroU8");
    let TypeDef::Struct(struct_def) = &ty.def else {
        panic!("NonZeroU8 should be a struct wrapper, got {:?}", ty.def);
    };
    assert_eq!(struct_def.fields.len(), 1);
    assert!(struct_def.fields[0].name.is_none());
    assert_eq!(
        struct_def.fields[0].type_decl,
        TypeDecl::Primitive(PrimitiveType::U8)
    );
}
