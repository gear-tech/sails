use sails_idl_ast::{StructDef, TypeDef};
use sails_type_registry::{Registry, TypeDecl, TypeInfo};

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
    let struct_decl = registry.get_type_decl(struct_ref).unwrap();

    match struct_decl {
        TypeDecl::Named { name, generics, .. } => {
            assert_eq!(name, "MultiGen");
            assert_eq!(generics.len(), 2);
        }
        _ => panic!("Expected Named type"),
    }

    // Check type_def has type_params
    let struct_def = registry.get_type(struct_ref).unwrap();
    assert_eq!(struct_def.type_params.len(), 2);
    assert_eq!(struct_def.type_params[0].name, "T1");
    assert_eq!(struct_def.type_params[1].name, "T2");

    // Check enum MultiEnum<bool, u64, i32>
    let enum_ref = registry.register_type::<MultiEnum<bool, u64, i32>>();
    let enum_decl = registry.get_type_decl(enum_ref).unwrap();

    match enum_decl {
        TypeDecl::Named { name, generics, .. } => {
            assert_eq!(name, "MultiEnum");
            assert_eq!(generics.len(), 3);
        }
        _ => panic!("Expected Named type"),
    }

    let enum_def = registry.get_type(enum_ref).unwrap();
    assert_eq!(enum_def.type_params.len(), 3);
    assert_eq!(enum_def.type_params[0].name, "A");
    assert_eq!(enum_def.type_params[1].name, "B");
    assert_eq!(enum_def.type_params[2].name, "C");
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
    let struct_decl = registry.get_type_decl(struct_ref).unwrap();

    match struct_decl {
        TypeDecl::Named { name, generics, .. } => {
            assert_eq!(name, "ConstGen");
            assert_eq!(generics.len(), 1);
        }
        _ => panic!("Expected Named type"),
    }

    let struct_def = registry.get_type(struct_ref).unwrap();
    assert_eq!(struct_def.type_params.len(), 1);
    assert_eq!(struct_def.type_params[0].name, "N");

    // Check field is an array with length 32
    if let TypeDef::Struct(StructDef { fields }) = &struct_def.def {
        assert_eq!(fields.len(), 1);
        match &fields[0].type_decl {
            TypeDecl::Array { len, .. } => assert_eq!(*len, 32),
            _ => panic!("Expected Array type for field"),
        }
    } else {
        panic!("Expected Struct definition");
    }
}

#[test]
fn test_template_preserves_generic_parameters() {
    use sails_idl_ast::TypeDecl as AstTypeDecl;

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct MyStruct<T> {
        field: T,
    }

    let mut registry = Registry::new();

    // Register MyStruct<bool> - this should create a template MyStruct<T>
    let _ = registry.register_type::<MyStruct<bool>>();

    // Now check the template (type_def)
    // Find the type by name
    let template = registry
        .named_types()
        .find(|t| t.name == "MyStruct")
        .expect("MyStruct template not found");

    // Template should have type_params
    assert_eq!(template.type_params.len(), 1);
    assert_eq!(template.type_params[0].name, "T");
    assert!(
        template.type_params[0].ty.is_none(),
        "Template parameter should have no concrete type"
    );

    // Check struct fields
    if let TypeDef::Struct(StructDef { fields }) = &template.def {
        assert_eq!(fields.len(), 1);
        // Field type should be a parameter reference, not bool
        match &fields[0].type_decl {
            AstTypeDecl::Named { name, param, .. } => {
                assert_eq!(name, "T");
                assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
            }
            _ => panic!(
                "Expected parameter reference, got {:?}",
                fields[0].type_decl
            ),
        }
    } else {
        panic!("Expected Struct definition");
    }
}

#[test]
fn test_mixed_types_in_template() {
    use sails_idl_ast::TypeDecl as AstTypeDecl;

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct MixedStruct<T> {
        generic_field: T,
        concrete_field: u32,
        tuple_field: (T, String),
    }

    let mut registry = Registry::new();

    // Register MixedStruct<bool>
    let _ = registry.register_type::<MixedStruct<bool>>();

    // Find the template
    let template = registry
        .named_types()
        .find(|t| t.name == "MixedStruct")
        .expect("MixedStruct template not found");

    // Check struct fields
    if let TypeDef::Struct(StructDef { fields }) = &template.def {
        assert_eq!(fields.len(), 3);

        // generic_field should be parameter T
        match &fields[0].type_decl {
            AstTypeDecl::Named { name, param, .. } => {
                assert_eq!(name, "T");
                assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
            }
            _ => panic!("Expected parameter for generic_field"),
        }

        // concrete_field should be u32
        match &fields[1].type_decl {
            AstTypeDecl::Primitive(p) => {
                assert_eq!(*p, sails_idl_ast::PrimitiveType::U32);
            }
            _ => panic!("Expected primitive for concrete_field"),
        }

        // tuple_field should be (T, String)
        match &fields[2].type_decl {
            AstTypeDecl::Tuple { types } => {
                assert_eq!(types.len(), 2);
                // First element should be T
                match &types[0] {
                    AstTypeDecl::Named { name, param, .. } => {
                        assert_eq!(name, "T");
                        assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                    }
                    _ => panic!("Expected T in tuple"),
                }
                // Second element should be String
                match &types[1] {
                    AstTypeDecl::Primitive(p) => {
                        assert_eq!(*p, sails_idl_ast::PrimitiveType::String);
                    }
                    _ => panic!("Expected String in tuple"),
                }
            }
            _ => panic!("Expected tuple for tuple_field"),
        }
    } else {
        panic!("Expected Struct definition");
    }
}
