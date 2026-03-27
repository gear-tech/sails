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

    // Verify it's an Applied L3<T, E>
    if let TypeDef::Applied { base, args } = &field_ty.def {
        assert_eq!(args.len(), 2);

        // base is Result<Vec<(T, u8)>, E>
        let base_ty = registry.get_type(*base).unwrap();
        assert!(matches!(base_ty.def, TypeDef::Result { .. }));

        // The arguments applied to the base are the generic parameters T and E
        let t_param = registry.get_type(args[0]).unwrap();
        assert!(matches!(t_param.def, TypeDef::Parameter(ref name) if name == "T"));

        let e_param = registry.get_type(args[1]).unwrap();
        assert!(matches!(e_param.def, TypeDef::Parameter(ref name) if name == "E"));
    } else {
        panic!("Expected Applied type for alias, got {:?}", field_ty.def);
    }
}
