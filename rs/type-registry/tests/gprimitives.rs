use gprimitives::{ActorId, CodeId, H160, H256, MessageId, NonZeroU256, U256};
use sails_idl_ast::{PrimitiveType, StructDef, TypeDecl, TypeDef};
use sails_type_registry::Registry;

#[test]
fn test_gear_primitives() {
    let mut registry = Registry::new();

    let actor_id_ref = registry.register_type::<ActorId>();
    let actor_id_decl = registry.get_type_decl(actor_id_ref).unwrap();
    assert!(matches!(
        actor_id_decl,
        TypeDecl::Primitive(PrimitiveType::ActorId)
    ));

    let message_id_ref = registry.register_type::<MessageId>();
    let message_id_decl = registry.get_type_decl(message_id_ref).unwrap();
    assert!(matches!(
        message_id_decl,
        TypeDecl::Primitive(PrimitiveType::MessageId)
    ));

    let code_id_ref = registry.register_type::<CodeId>();
    let code_id_decl = registry.get_type_decl(code_id_ref).unwrap();
    assert!(matches!(
        code_id_decl,
        TypeDecl::Primitive(PrimitiveType::CodeId)
    ));

    let h160_ref = registry.register_type::<H160>();
    let h160_decl = registry.get_type_decl(h160_ref).unwrap();
    assert!(matches!(
        h160_decl,
        TypeDecl::Primitive(PrimitiveType::H160)
    ));

    let h256_ref = registry.register_type::<H256>();
    let h256_decl = registry.get_type_decl(h256_ref).unwrap();
    assert!(matches!(
        h256_decl,
        TypeDecl::Primitive(PrimitiveType::H256)
    ));

    let u256_ref = registry.register_type::<U256>();
    let u256_decl = registry.get_type_decl(u256_ref).unwrap();
    assert!(matches!(
        u256_decl,
        TypeDecl::Primitive(PrimitiveType::U256)
    ));
}

#[test]
fn test_nonzero_u256_is_composite() {
    let mut registry = Registry::new();

    let id = registry.register_type::<NonZeroU256>();
    let ty = registry.get_type(id).expect("Type should be in registry");

    if let TypeDef::Struct(StructDef { fields }) = &ty.def {
        assert_eq!(fields.len(), 1);
        match &fields[0].type_decl {
            TypeDecl::Primitive(p) => {
                assert_eq!(*p, PrimitiveType::U256);
            }
            other => panic!(
                "NonZeroU256 field should be U256 primitive, got {:?}",
                other
            ),
        }
    } else {
        panic!("NonZeroU256 should be struct, got {:?}", ty.def);
    }
    assert_eq!(ty.name, "NonZeroU256");
}
