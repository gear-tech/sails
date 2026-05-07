use alloc::string::String;
use sails_idl_ast::{PrimitiveType, TypeDecl, TypeDef};
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn multiple_type_generics_are_captured_as_abstract_params() {
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

    let struct_ref = registry.register_type::<MultiGen<u32, String>>().unwrap();
    let struct_ty = registry.get_type(struct_ref).unwrap();

    assert_eq!(struct_ty.type_params.len(), 2);
    assert_eq!(struct_ty.type_params[0].name, "T1");
    assert!(struct_ty.type_params[0].ty.is_none());
    assert_eq!(struct_ty.type_params[1].name, "T2");
    assert!(struct_ty.type_params[1].ty.is_none());

    let TypeDef::Struct(struct_def) = &struct_ty.def else {
        panic!("expected struct");
    };
    assert_eq!(struct_def.fields[0].type_decl, TypeDecl::generic("T1"));
    assert_eq!(struct_def.fields[1].type_decl, TypeDecl::generic("T2"));

    let enum_ref = registry
        .register_type::<MultiEnum<bool, u64, i32>>()
        .unwrap();
    let enum_ty = registry.get_type(enum_ref).unwrap();

    assert_eq!(enum_ty.type_params.len(), 3);
    assert_eq!(enum_ty.type_params[0].name, "A");
    assert_eq!(enum_ty.type_params[1].name, "B");
    assert_eq!(enum_ty.type_params[2].name, "C");
}

#[test]
fn const_generics_encode_value_into_name_suffix() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct ConstGen<const N: usize> {
        data: [u8; N],
    }

    let mut registry = Registry::new();
    let struct_ref = registry.register_type::<ConstGen<32>>().unwrap();
    let struct_ty = registry.get_type(struct_ref).unwrap();

    assert_eq!(struct_ty.name, "ConstGenN32");
    assert!(struct_ty.type_params.is_empty());

    let TypeDef::Struct(struct_def) = &struct_ty.def else {
        panic!("expected struct");
    };

    assert_eq!(
        struct_def.fields[0].type_decl,
        TypeDecl::Array {
            item: alloc::boxed::Box::new(TypeDecl::Primitive(PrimitiveType::U8)),
            len: 32,
        }
    );
}
