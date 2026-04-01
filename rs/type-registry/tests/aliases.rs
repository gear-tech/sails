use alloc::string::String;
use alloc::vec::Vec;
use sails_type_registry::alloc;
use sails_type_registry::ty::TypeDef;
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn test_deep_alias_recursion_expansion() {
    type L1<T> = (T, u8);
    type L2<T> = Vec<L1<T>>;
    type L3<T, E> = Result<L2<T>, E>;

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Deep<T, E> {
        field: L3<T, E>,
    }

    let mut registry = Registry::new();

    let _ = registry.register_type::<u32>();
    let _ = registry.register_type::<u8>();
    let _string_id = registry.register_type::<String>();

    let deep_ref = registry.register_type::<Deep<u32, String>>();
    let deep_ty = registry.get_type(deep_ref).expect("Deep type not found");

    // In the new world, everything is in `def`.
    let TypeDef::Composite(comp) = &deep_ty.def else {
        panic!()
    };
    let field_ty_id = comp.fields[0].ty;
    let field_ty = registry.get_type(field_ty_id).unwrap();

    // Verify it's normalized to Result (from L3)
    if let TypeDef::Result { ok, err } = &field_ty.def {
        // Since we are looking at the generic definition of Deep<T, E>,
        // the field type L3<T, E> is resolved to its base Result<L2<T>, E>.
        // In the context of Deep, the arguments to L3 are its own T and E.

        let ok_ty = registry.get_type(*ok).unwrap();
        // L2<T> is the first arg of L3. In the normalized Result, it's represented as a Parameter
        assert!(matches!(ok_ty.def, TypeDef::Parameter(ref name) if name == "T"));

        let err_ty = registry.get_type(*err).unwrap();
        assert!(matches!(err_ty.def, TypeDef::Parameter(ref name) if name == "E"));
    } else {
        panic!("Expected Result type for alias, got {:?}", field_ty.def);
    }
}
