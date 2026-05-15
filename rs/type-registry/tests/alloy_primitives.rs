use alloy_primitives::{Address, B256};
use sails_idl_ast::{PrimitiveType, TypeDecl};
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn alloy_primitives_lower_to_built_in_primitive_type_decl() {
    let mut registry = Registry::new();

    assert_eq!(
        <Address as TypeInfo>::type_decl(&mut registry),
        TypeDecl::Primitive(PrimitiveType::H160),
    );
    assert_eq!(
        <B256 as TypeInfo>::type_decl(&mut registry),
        TypeDecl::Primitive(PrimitiveType::H256),
    );

    assert!(
        registry.is_empty(),
        "alloy primitives must not allocate registry slots"
    );
}
