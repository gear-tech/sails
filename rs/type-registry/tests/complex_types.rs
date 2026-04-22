use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use sails_idl_ast::{PrimitiveType, TypeDecl, TypeDef};
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn deep_collection_fields_lower_abstractly() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct DeepGraph {
        _nodes: BTreeMap<u32, Vec<Option<String>>>,
    }

    let mut registry = Registry::new();
    let graph_ref = registry.register_type::<DeepGraph>().unwrap();
    let graph_ty = registry.get_type(graph_ref).unwrap();

    assert_eq!(graph_ty.name, "DeepGraph");

    let TypeDef::Struct(struct_def) = &graph_ty.def else {
        panic!("expected struct");
    };

    let expected = TypeDecl::Slice {
        item: Box::new(TypeDecl::Tuple {
            types: alloc::vec![
                TypeDecl::Primitive(PrimitiveType::U32),
                TypeDecl::Slice {
                    item: Box::new(TypeDecl::Named {
                        name: "Option".into(),
                        generics: alloc::vec![TypeDecl::Primitive(PrimitiveType::String)],
                    }),
                },
            ],
        }),
    };
    assert_eq!(struct_def.fields[0].type_decl, expected);
}

#[test]
fn self_recursive_type_stores_named_decl_to_itself() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct LinkedList {
        _value: u32,
        _next: Option<Box<LinkedList>>,
    }

    let mut registry = Registry::new();
    let list_ref = registry.register_type::<LinkedList>().unwrap();
    let list_ty = registry.get_type(list_ref).unwrap();

    assert_eq!(list_ty.name, "LinkedList");

    let TypeDef::Struct(struct_def) = &list_ty.def else {
        panic!("expected struct");
    };
    assert_eq!(struct_def.fields.len(), 2);

    let TypeDecl::Named { name, generics } = &struct_def.fields[1].type_decl else {
        panic!("expected Option<LinkedList>");
    };
    assert_eq!(name, "Option");
    assert_eq!(generics.len(), 1);

    let TypeDecl::Named { name, generics } = &generics[0] else {
        panic!("expected inner Named");
    };
    assert_eq!(name, "LinkedList");
    assert!(generics.is_empty());
}

#[test]
fn mutually_recursive_types_register_both_sides() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Ping {
        _pong: Option<Box<Pong>>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Pong {
        _ping: Option<Box<Ping>>,
    }

    let mut registry = Registry::new();

    let ping_ref = registry.register_type::<Ping>().unwrap();
    let pong_ref = registry.register_type::<Pong>().unwrap();
    assert_ne!(ping_ref, pong_ref);

    let ping_ty = registry.get_type(ping_ref).unwrap();
    let TypeDef::Struct(struct_def) = &ping_ty.def else {
        panic!("expected struct");
    };

    let TypeDecl::Named {
        name: outer_name,
        generics: outer_generics,
    } = &struct_def.fields[0].type_decl
    else {
        panic!("expected Option<Pong>");
    };
    assert_eq!(outer_name, "Option");

    let TypeDecl::Named {
        name: inner_name, ..
    } = &outer_generics[0]
    else {
        panic!("expected inner Named Pong");
    };
    assert_eq!(inner_name, "Pong");
}

#[test]
fn option_of_generic_param_stored_abstractly() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Wrapper<T> {
        _inner: Option<T>,
    }

    let mut registry = Registry::new();
    let wrapper_ref = registry.register_type::<Wrapper<u32>>().unwrap();
    let wrapper_ty = registry.get_type(wrapper_ref).unwrap();

    let TypeDef::Struct(struct_def) = &wrapper_ty.def else {
        panic!("expected struct");
    };
    assert_eq!(
        struct_def.fields[0].type_decl,
        TypeDecl::Named {
            name: "Option".into(),
            generics: alloc::vec![TypeDecl::generic("T")],
        }
    );
}

#[test]
fn const_generic_struct_suffixes_name_and_substitutes_length() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct ConstWrapper<T, const N: usize> {
        _data: [T; N],
        _nested: [Option<T>; N],
    }

    let mut registry = Registry::new();
    let wrapper_ref = registry.register_type::<ConstWrapper<u32, 10>>().unwrap();
    let wrapper_ty = registry.get_type(wrapper_ref).unwrap();

    assert_eq!(wrapper_ty.name, "ConstWrapperN10");
    assert_eq!(wrapper_ty.type_params.len(), 1);
    assert_eq!(wrapper_ty.type_params[0].name, "T");

    let TypeDef::Struct(struct_def) = &wrapper_ty.def else {
        panic!("expected struct");
    };
    assert_eq!(struct_def.fields.len(), 2);

    assert_eq!(
        struct_def.fields[0].type_decl,
        TypeDecl::Array {
            item: Box::new(TypeDecl::generic("T")),
            len: 10,
        }
    );
    assert_eq!(
        struct_def.fields[1].type_decl,
        TypeDecl::Array {
            item: Box::new(TypeDecl::Named {
                name: "Option".into(),
                generics: alloc::vec![TypeDecl::generic("T")],
            }),
            len: 10,
        }
    );
}
