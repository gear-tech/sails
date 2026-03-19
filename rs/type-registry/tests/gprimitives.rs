extern crate alloc;

use gprimitives::{ActorId, CodeId, H160, H256, MessageId, U256};
use sails_type_registry::{
    Registry,
    ty::{GPrimitive, TypeDef, TypeDefinitionKind},
};

macro_rules! assert_gprimitive {
    ($registry:expr, $t:ty, $expected_primitive:ident) => {
        let id = $registry.register_type::<$t>();
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
fn test_gear_primitives() {
    let mut registry = Registry::new();

    assert_gprimitive!(registry, ActorId, ActorId);
    assert_gprimitive!(registry, MessageId, MessageId);
    assert_gprimitive!(registry, CodeId, CodeId);
    assert_gprimitive!(registry, H160, H160);
    assert_gprimitive!(registry, H256, H256);
    assert_gprimitive!(registry, U256, U256);
}

#[test]
fn test_nonzero_u256_is_composite() {
    use gprimitives::NonZeroU256;
    let mut registry = Registry::new();

    let id = registry.register_type::<NonZeroU256>();
    let ty = registry.get_type(id).expect("Type should be in registry");

    if let TypeDef::Definition(def) = &ty.def
        && let TypeDefinitionKind::Composite(c) = &def.kind
    {
        assert_eq!(c.fields.len(), 1);
        assert!(registry.is_type::<U256>(c.fields[0].ty.id()));
    } else {
        panic!("NonZeroU256 should be composite, got {:?}", ty.def);
    }
    assert_eq!(ty.name, "NonZeroU256");
}
