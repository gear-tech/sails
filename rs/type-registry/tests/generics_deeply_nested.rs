use sails_idl_ast::{StructDef, TypeDecl as AstTypeDecl, TypeDef};
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn test_deeply_nested_generics_in_template() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct DeepNested<T1, T2> {
        field1: Vec<Option<Result<T1, T2>>>,
        field2: (T1, [T2; 4], Vec<T1>),
        field3: Option<Vec<(T1, T2)>>,
        field4: T1,
        field5: u64,
    }

    let mut registry = Registry::new();

    // Register DeepNested<String, u32>
    let _ = registry.register_type::<DeepNested<String, u32>>();

    // Find the template
    let template = registry
        .named_types()
        .find(|t| t.name == "DeepNested")
        .expect("DeepNested template not found");

    // Template should have 2 type_params
    assert_eq!(template.type_params.len(), 2);
    assert_eq!(template.type_params[0].name, "T1");
    assert_eq!(template.type_params[1].name, "T2");

    // Check struct fields
    if let TypeDef::Struct(StructDef { fields }) = &template.def {
        assert_eq!(fields.len(), 5);

        // field1: Vec<Option<Result<T1, T2>>>
        match &fields[0].type_decl {
            AstTypeDecl::Slice { item } => {
                match item.as_ref() {
                    AstTypeDecl::Named { name, generics, .. } => {
                        assert_eq!(name, "Option");
                        assert_eq!(generics.len(), 1);
                        match &generics[0] {
                            AstTypeDecl::Named { name, generics, .. } => {
                                assert_eq!(name, "Result");
                                assert_eq!(generics.len(), 2);
                                // T1
                                match &generics[0] {
                                    AstTypeDecl::Named { name, param, .. } => {
                                        assert_eq!(name, "T1");
                                        assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                                    }
                                    other => panic!("Expected T1 parameter, got {:?}", other),
                                }
                                // T2
                                match &generics[1] {
                                    AstTypeDecl::Named { name, param, .. } => {
                                        assert_eq!(name, "T2");
                                        assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                                    }
                                    other => panic!("Expected T2 parameter, got {:?}", other),
                                }
                            }
                            other => panic!("Expected Result inside Option, got {:?}", other),
                        }
                    }
                    other => panic!("Expected Option inside Vec, got {:?}", other),
                }
            }
            other => panic!("Expected Vec (Slice) for field1, got {:?}", other),
        }

        // field2: (T1, [T2; 4], Vec<T1>)
        match &fields[1].type_decl {
            AstTypeDecl::Tuple { types } => {
                assert_eq!(types.len(), 3);
                // T1
                match &types[0] {
                    AstTypeDecl::Named { name, param, .. } => {
                        assert_eq!(name, "T1");
                        assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                    }
                    _ => panic!("Expected T1 in tuple"),
                }
                // [T2; 4]
                match &types[1] {
                    AstTypeDecl::Array { item, len } => {
                        assert_eq!(*len, 4);
                        match item.as_ref() {
                            AstTypeDecl::Named { name, param, .. } => {
                                assert_eq!(name, "T2");
                                assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                            }
                            _ => panic!("Expected T2 in array"),
                        }
                    }
                    _ => panic!("Expected array in tuple"),
                }
                // Vec<T1>
                match &types[2] {
                    AstTypeDecl::Slice { item } => match item.as_ref() {
                        AstTypeDecl::Named { name, param, .. } => {
                            assert_eq!(name, "T1");
                            assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                        }
                        _ => panic!("Expected T1 in Vec"),
                    },
                    _ => panic!("Expected Vec in tuple"),
                }
            }
            _ => panic!("Expected tuple for field2"),
        }

        // field3: Option<Vec<(T1, T2)>>
        match &fields[2].type_decl {
            AstTypeDecl::Named { name, generics, .. } => {
                assert_eq!(name, "Option");
                assert_eq!(generics.len(), 1);
                match &generics[0] {
                    AstTypeDecl::Slice { item } => {
                        match item.as_ref() {
                            AstTypeDecl::Tuple { types } => {
                                assert_eq!(types.len(), 2);
                                // T1
                                match &types[0] {
                                    AstTypeDecl::Named { name, param, .. } => {
                                        assert_eq!(name, "T1");
                                        assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                                    }
                                    _ => panic!("Expected T1"),
                                }
                                // T2
                                match &types[1] {
                                    AstTypeDecl::Named { name, param, .. } => {
                                        assert_eq!(name, "T2");
                                        assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                                    }
                                    _ => panic!("Expected T2"),
                                }
                            }
                            _ => panic!("Expected tuple"),
                        }
                    }
                    _ => panic!("Expected Vec inside Option"),
                }
            }
            _ => panic!("Expected Option for field3"),
        }

        // field4: T1
        match &fields[3].type_decl {
            AstTypeDecl::Named { name, param, .. } => {
                assert_eq!(name, "T1");
                assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
            }
            _ => panic!("Expected T1 parameter for field4"),
        }

        // field5: u64
        match &fields[4].type_decl {
            AstTypeDecl::Primitive(p) => {
                assert_eq!(*p, sails_idl_ast::PrimitiveType::U64);
            }
            _ => panic!("Expected u64 primitive for field5"),
        }
    } else {
        panic!("Expected Struct definition");
    }
}
