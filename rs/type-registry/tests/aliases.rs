extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use sails_type_registry::ty::{FieldType, TypeDef};
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

    let u32_id = registry.register_type::<u32>();
    let u8_id = registry.register_type::<u8>();
    let string_id = registry.register_type::<String>();

    let deep_ref = registry.register_type::<Deep<u32, String>>();
    let deep_ty = registry.get_type(deep_ref).expect("Deep type not found");

    // Chair 1: Pure template for idl-gen
    let TypeDef::Composite(comp) = &deep_ty.def else {
        panic!()
    };
    let field = &comp.fields[0];
    // In Chair 1, it's just a raw Result alias (L3) with parameters T and E
    let FieldType::Parameterized { id: l3_id, args } = &field.ty else {
        panic!()
    };
    assert_eq!(args[0], FieldType::Parameter("T".to_string()));
    assert_eq!(args[1], FieldType::Parameter("E".to_string()));
    let l3_ty = registry.get_type(*l3_id).unwrap();
    assert!(matches!(l3_ty.def, TypeDef::Result { .. }));

    // Chair 2: Fully expanded definition for deep analysis
    let TypeDef::Composite(exp_comp) = deep_ty
        .expanded_def
        .as_ref()
        .expect("Expected expanded_def")
    else {
        panic!()
    };
    let exp_field = &exp_comp.fields[0];

    // Verify deep expansion: Result<Vec<(u32, u8)>, String>
    let FieldType::Parameterized {
        id: res_id,
        args: res_args,
    } = &exp_field.ty
    else {
        panic!(
            "Expected Parameterized field for L3 in expanded_def, got {:?}",
            exp_field.ty
        )
    };

    let res_ty = registry.get_type(*res_id).unwrap();
    assert!(matches!(res_ty.def, TypeDef::Result { .. }));
    assert_eq!(res_args.len(), 2);

    // Arg 0 of Result: Vec<(u32, u8)>
    let FieldType::Parameterized { args: vec_args, .. } = &res_args[0] else {
        panic!(
            "Expected Parameterized field for L2 (Vec), got {:?}",
            res_args[0]
        )
    };

    // Inside Vec: Tuple(u32, u8)
    let FieldType::Tuple {
        elems: tuple_elems, ..
    } = &vec_args[0]
    else {
        panic!(
            "Expected Tuple field for L1 expansion, got {:?}",
            vec_args[0]
        )
    };

    assert_eq!(tuple_elems[0], FieldType::Id(u32_id));
    assert_eq!(tuple_elems[1], FieldType::Id(u8_id));

    // Arg 1 of Result: String
    assert_eq!(res_args[1], FieldType::Id(string_id));
}
