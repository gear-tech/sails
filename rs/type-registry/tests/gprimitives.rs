use gprimitives::{ActorId, CodeId, H160, H256, MessageId, U256};
use sails_idl_ast::{PrimitiveType, TypeDecl, TypeDef};
use sails_type_registry::{Registry, TypeInfo};

macro_rules! assert_gprimitive {
    ($registry:expr, $ty:ty, $expected:ident) => {
        assert_eq!(
            <$ty as TypeInfo>::type_decl(&mut $registry),
            TypeDecl::Primitive(PrimitiveType::$expected),
        );
    };
}

#[test]
fn gear_primitives_lower_to_built_in_primitive_type_decl() {
    let mut registry = Registry::new();

    assert_gprimitive!(registry, ActorId, ActorId);
    assert_gprimitive!(registry, MessageId, MessageId);
    assert_gprimitive!(registry, CodeId, CodeId);
    assert_gprimitive!(registry, H160, H160);
    assert_gprimitive!(registry, H256, H256);
    assert_gprimitive!(registry, U256, U256);

    assert!(
        registry.is_empty(),
        "gear primitives must not allocate registry slots"
    );
}

#[test]
fn non_zero_u256_registers_as_named_wrapper() {
    use gprimitives::NonZeroU256;

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<NonZeroU256>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert_eq!(ty.name, "NonZeroU256");
    let TypeDef::Struct(struct_def) = &ty.def else {
        panic!("NonZeroU256 should be a struct wrapper, got {:?}", ty.def);
    };
    assert_eq!(struct_def.fields.len(), 1);
    assert!(struct_def.fields[0].name.is_none());
    assert_eq!(
        struct_def.fields[0].type_decl,
        TypeDecl::Primitive(PrimitiveType::U256)
    );
}
