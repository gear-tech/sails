use alloy_primitives::{Address, B256};
use sails_idl_ast::{PrimitiveType, TypeDecl};
use sails_type_registry::Registry;

macro_rules! assert_alloy_primitive {
    ($registry:expr, $ty:ty, $expected_primitive:ident) => {
        let id = $registry.register_type::<$ty>();
        let type_decl = $registry
            .get_type_decl(id)
            .expect("Type declaration should be in registry");
        assert_eq!(
            type_decl,
            &TypeDecl::Primitive(PrimitiveType::$expected_primitive)
        );
        // Primitives don't have a full Type definition (only TypeDecl), so get_type returns None
        assert!(
            $registry.get_type(id).is_none(),
            "Primitives should not have a full Type definition"
        );
    };
}

#[test]
fn registers_alloy_primitives_as_existing_idl_primitives() {
    let mut registry = Registry::new();

    assert_alloy_primitive!(registry, Address, H160);
    assert_alloy_primitive!(registry, B256, H256);
}
