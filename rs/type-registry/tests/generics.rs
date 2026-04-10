use alloc::string::String;
use sails_type_registry::alloc;
use sails_type_registry::ty::{GenericArg, TypeDef};
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn test_multiple_generics() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct MultiGen<T1, T2> {
        _a: T1,
        _b: T2,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum MultiEnum<A, B, C> {
        _V1(A),
        _V2(B, C),
    }

    let mut registry = Registry::new();

    // Check struct MultiGen<u32, String>
    let struct_ref = registry.register_type::<MultiGen<u32, String>>();
    let struct_ty = registry.get_type(struct_ref).unwrap();

    assert_eq!(struct_ty.type_params.len(), 2);
    assert_eq!(struct_ty.type_params[0].name, "T1");
    match &struct_ty.type_params[0].arg {
        GenericArg::Type(_) => (),
        _ => panic!("Expected Type argument for T1"),
    }
    assert_eq!(struct_ty.type_params[1].name, "T2");
    match &struct_ty.type_params[1].arg {
        GenericArg::Type(_) => (),
        _ => panic!("Expected Type argument for T2"),
    }

    // Check enum MultiEnum<bool, u64, i32>
    let enum_ref = registry.register_type::<MultiEnum<bool, u64, i32>>();
    let enum_ty = registry.get_type(enum_ref).unwrap();

    assert_eq!(enum_ty.type_params.len(), 3);
    assert_eq!(enum_ty.type_params[0].name, "A");
    assert_eq!(enum_ty.type_params[1].name, "B");
    assert_eq!(enum_ty.type_params[2].name, "C");
}

#[test]
fn test_const_generics() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct ConstGen<const N: usize> {
        data: [u8; N],
    }

    let mut registry = Registry::new();
    let struct_ref = registry.register_type::<ConstGen<32>>();
    let struct_ty = registry.get_type(struct_ref).unwrap();

    assert_eq!(struct_ty.type_params.len(), 1);
    assert_eq!(struct_ty.type_params[0].name, "N");
    match &struct_ty.type_params[0].arg {
        GenericArg::Const(val) => assert_eq!(val, "32"),
        _ => panic!("Expected Const argument for N"),
    }

    if let TypeDef::Composite(comp) = &struct_ty.def {
        let field_ty_ref = comp.fields[0].ty;
        let field_ty = registry.get_type(field_ty_ref).unwrap();

        match &field_ty.def {
            TypeDef::Array { len, .. } => assert_eq!(*len, 32),
            _ => panic!("Expected Array definition for field data"),
        }
    } else {
        panic!("Expected Composite definition");
    }
}
