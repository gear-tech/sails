use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use sails_idl_ast::{PrimitiveType, TypeDecl, TypeDef};
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn deeply_nested_containers_lower_into_substituted_type_decl() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Deep<T, E> {
        field: Result<Vec<(T, u8)>, E>,
    }

    let mut registry = Registry::new();
    let deep_ref = registry.register_type::<Deep<u32, String>>().unwrap();
    let deep_ty = registry.get_type(deep_ref).unwrap();

    let TypeDef::Struct(struct_def) = &deep_ty.def else {
        panic!("expected struct");
    };

    // The abstract stored def keeps T and E as generic references; it is shared
    // across all concrete instantiations.
    let expected_field_decl = TypeDecl::Named {
        name: "Result".into(),
        generics: alloc::vec![
            TypeDecl::Slice {
                item: Box::new(TypeDecl::Tuple {
                    types: alloc::vec![
                        TypeDecl::generic("T"),
                        TypeDecl::Primitive(PrimitiveType::U8),
                    ],
                }),
            },
            TypeDecl::generic("E"),
        ],
    };
    assert_eq!(struct_def.fields[0].type_decl, expected_field_decl);
    assert_eq!(deep_ty.type_params.len(), 2);
    assert_eq!(deep_ty.type_params[0].name, "T");
    assert_eq!(deep_ty.type_params[1].name, "E");
}
