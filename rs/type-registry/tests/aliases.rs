use alloc::string::String;
use alloc::vec::Vec;
use sails_idl_ast::{StructDef, TypeDef};
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeDecl, TypeInfo};

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

    // Check that Deep has type_params
    assert_eq!(deep_ty.type_params.len(), 2);
    assert_eq!(deep_ty.type_params[0].name, "T");
    assert_eq!(deep_ty.type_params[1].name, "E");

    // Check field type_decl
    if let TypeDef::Struct(StructDef { fields }) = &deep_ty.def {
        assert_eq!(fields.len(), 1);

        // field: L3<T, E> should be Named { name: "L3", generics: [T_param, E_param] }
        match &fields[0].type_decl {
            TypeDecl::Named { name, generics, .. } => {
                assert_eq!(name, "L3");
                assert_eq!(generics.len(), 2);
                // Check generics are parameters
                match &generics[0] {
                    TypeDecl::Named { name, param, .. } => {
                        assert_eq!(name, "T");
                        assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                    }
                    other => panic!("Expected T parameter, got {:?}", other),
                }
                match &generics[1] {
                    TypeDecl::Named { name, param, .. } => {
                        assert_eq!(name, "E");
                        assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
                    }
                    other => panic!("Expected E parameter, got {:?}", other),
                }
            }
            other => panic!("Expected L3 named type, got {:?}", other),
        }
    } else {
        panic!("Expected Struct definition, got {:?}", deep_ty.def);
    }
}
