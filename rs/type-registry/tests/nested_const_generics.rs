use sails_idl_ast::{StructDef, TypeDecl as AstTypeDecl, TypeDef};
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn test_nested_const_generics() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Wrapper<T, const N: usize> {
        value: T,
        bytes: [u8; N],
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Holder<T, const N: usize> {
        inner: Wrapper<T, N>,
        maybe: Option<Wrapper<T, N>>,
    }

    let mut registry = Registry::new();
    let holder_ref = registry.register_type::<Holder<u32, 16>>();

    // Check Holder type_def
    let holder_def = registry.get_type(holder_ref).unwrap();

    if let TypeDef::Struct(StructDef { fields }) = &holder_def.def {
        assert_eq!(fields.len(), 2);

        // Check inner field: Wrapper<T, N>
        match &fields[0].type_decl {
            AstTypeDecl::Named { name, generics, .. } => {
                eprintln!("Wrapper generics: {:#?}", generics);
                assert_eq!(name, "Wrapper");
                assert_eq!(
                    generics.len(),
                    2,
                    "Wrapper should have 2 generic parameters"
                );

                // First generic is T parameter
                match &generics[0] {
                    AstTypeDecl::Named { name, param, .. } => {
                        assert_eq!(name, "T");
                        assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                    }
                    other => panic!("Expected T parameter, got {:?}", other),
                }

                // Second generic is N const parameter with value 16
                match &generics[1] {
                    AstTypeDecl::Named { name, param, .. } => {
                        assert_eq!(name, "N");
                        assert_eq!(
                            param,
                            &Some(sails_idl_ast::NamedParam::Const {
                                value: "16".to_string(),
                            })
                        );
                    }
                    other => panic!("Expected N const parameter with value 16, got {:?}", other),
                }
            }
            other => panic!("Expected Wrapper Named type, got {:?}", other),
        }

        // Check maybe field: Option<Wrapper<T, N>>
        match &fields[1].type_decl {
            AstTypeDecl::Named { name, generics, .. } => {
                assert_eq!(name, "Option");
                assert_eq!(generics.len(), 1);

                match &generics[0] {
                    AstTypeDecl::Named {
                        name: inner_name,
                        generics: inner_generics,
                        ..
                    } => {
                        assert_eq!(inner_name, "Wrapper");
                        assert_eq!(inner_generics.len(), 2);

                        // Check const parameter in nested Wrapper
                        match &inner_generics[1] {
                            AstTypeDecl::Named { name, param, .. } => {
                                assert_eq!(name, "N");
                                assert_eq!(
                                    param,
                                    &Some(sails_idl_ast::NamedParam::Const {
                                        value: "16".to_string(),
                                    })
                                );
                            }
                            other => panic!(
                                "Expected N const parameter in nested Wrapper, got {:?}",
                                other
                            ),
                        }
                    }
                    other => panic!("Expected Wrapper inside Option, got {:?}", other),
                }
            }
            other => panic!("Expected Option Named type, got {:?}", other),
        }
    } else {
        panic!("Expected Struct definition");
    }
}
