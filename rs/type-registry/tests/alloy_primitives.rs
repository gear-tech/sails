use alloy_primitives::{Address, B256};
use sails_type_registry::{
    Registry,
    ty::{GPrimitive, TypeDef},
};

macro_rules! assert_alloy_primitive {
    ($registry:expr, $ty:ty, $expected_primitive:ident) => {
        let id = $registry.register_type::<$ty>();
        let ty = $registry.get_type(id).expect("Type should be in registry");
        assert_eq!(ty.def, TypeDef::GPrimitive(GPrimitive::$expected_primitive));
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
fn registers_alloy_primitives_as_existing_idl_primitives() {
    let mut registry = Registry::new();

    assert_alloy_primitive!(registry, Address, H160);
    assert_alloy_primitive!(registry, B256, H256);
}
