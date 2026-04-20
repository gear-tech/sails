use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};
use sails_idl_ast::{StructDef, TypeDef};
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeDecl, TypeInfo};

#[test]
fn test_deep_collections() {
    #[derive(TypeInfo)]
    struct DeepGraph {
        _nodes: BTreeMap<u32, Vec<Option<String>>>,
    }

    let mut registry = Registry::new();

    let graph_ref = registry.register_type::<DeepGraph>();
    let graph_def = registry.get_type(graph_ref).unwrap();

    assert_eq!(graph_def.name, "DeepGraph");
    assert!(registry.len() >= 6);
}

#[test]
fn test_self_recursive_type() {
    #[derive(TypeInfo)]
    struct LinkedList {
        _value: u32,
        _next: Option<Box<LinkedList>>,
    }

    let mut registry = Registry::new();

    let list_ref = registry.register_type::<LinkedList>();
    let list_def = registry.get_type(list_ref).unwrap();

    assert_eq!(list_def.name, "LinkedList");

    if let TypeDef::Struct(StructDef { fields }) = &list_def.def {
        assert_eq!(fields.len(), 2);

        // Check _next field is Option<...>
        match &fields[1].type_decl {
            TypeDecl::Named { name, .. } => assert_eq!(name, "Option"),
            other => panic!("Expected Option, got {:?}", other),
        }
    } else {
        panic!("Expected Struct definition");
    }
}

#[test]
fn test_mutually_recursive_types() {
    #[derive(TypeInfo)]
    struct Ping {
        _pong: Option<Box<Pong>>,
    }

    #[derive(TypeInfo)]
    struct Pong {
        _ping: Option<Box<Ping>>,
    }

    let mut registry = Registry::new();

    let ping_ref = registry.register_type::<Ping>();
    let pong_ref = registry.register_type::<Pong>();

    assert!(ping_ref != pong_ref);

    let ping_def = registry.get_type(ping_ref).unwrap();
    if let TypeDef::Struct(StructDef { fields }) = &ping_def.def {
        // Check _pong field is Option<Pong>
        match &fields[0].type_decl {
            TypeDecl::Named { name, .. } => assert_eq!(name, "Option"),
            other => panic!("Expected Option, got {:?}", other),
        }
    } else {
        panic!("Expected Struct definition");
    }
}

#[test]
fn test_option_with_generic_param() {
    #[derive(TypeInfo)]
    struct Wrapper<T> {
        _inner: Option<T>,
    }

    let mut registry = Registry::new();
    let wrapper_ref = registry.register_type::<Wrapper<u32>>();
    let wrapper_def = registry.get_type(wrapper_ref).unwrap();

    if let TypeDef::Struct(StructDef { fields }) = &wrapper_def.def {
        // Check _inner field is Option<u32>
        match &fields[0].type_decl {
            TypeDecl::Named { name, .. } => assert_eq!(name, "Option"),
            other => panic!("Expected Option, got {:?}", other),
        }
    }
}

#[test]
fn test_const_generics_complex() {
    #[derive(TypeInfo)]
    struct ConstWrapper<T, const N: usize> {
        _data: [T; N],
        _nested: [Option<T>; N],
    }

    let mut registry = Registry::new();
    let wrapper_ref = registry.register_type::<ConstWrapper<u32, 10>>();
    let wrapper_def = registry.get_type(wrapper_ref).unwrap();

    assert_eq!(wrapper_def.type_params.len(), 2);
    assert_eq!(wrapper_def.type_params[0].name, "T");
    assert_eq!(wrapper_def.type_params[1].name, "N");

    if let TypeDef::Struct(StructDef { fields }) = &wrapper_def.def {
        assert_eq!(fields.len(), 2);

        // Check _data: [T; N]
        match &fields[0].type_decl {
            TypeDecl::Array { len, .. } => assert_eq!(*len, 10),
            other => panic!("Expected Array type, got {:?}", other),
        }

        // Check _nested: [Option<T>; N]
        match &fields[1].type_decl {
            TypeDecl::Array { len, .. } => assert_eq!(*len, 10),
            other => panic!("Expected Array type, got {:?}", other),
        }
    } else {
        panic!("Expected Struct definition");
    }
}
