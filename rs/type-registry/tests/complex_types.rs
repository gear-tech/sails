extern crate alloc;

use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};

use sails_type_registry::{Registry, TypeInfo, ty::TypeDef};

#[test]
fn test_deep_collections() {
    #[derive(TypeInfo)]
    struct DeepGraph {
        _nodes: BTreeMap<u32, Vec<Option<String>>>,
    }

    let mut registry = Registry::new();

    let graph_ref = registry.register_type::<DeepGraph>();
    let graph_ty = registry.get_type(graph_ref).unwrap();

    // Note: When defined inside a function, module_path!() will include the function name
    assert_eq!(graph_ty.name, "DeepGraph");
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
    let list_ty = registry.get_type(list_ref).unwrap();

    assert_eq!(list_ty.name, "LinkedList");

    if let TypeDef::Composite(c) = &list_ty.def {
        assert_eq!(c.fields.len(), 2);

        let next_opt_ref = c.fields[1].ty;
        let next_opt_ty = registry.get_type(next_opt_ref).unwrap();

        if let TypeDef::Option(inner_ref) = &next_opt_ty.def {
            assert_eq!(*inner_ref, list_ref);
        } else {
            panic!("Expected Option");
        }
    } else {
        panic!("Expected Composite, got {:?}", list_ty.def);
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

    let ping_ty = registry.get_type(ping_ref).unwrap();
    if let TypeDef::Composite(c) = &ping_ty.def {
        let pong_opt_ref = c.fields[0].ty;
        let pong_opt_ty = registry.get_type(pong_opt_ref).unwrap();

        if let TypeDef::Option(inner_ref) = &pong_opt_ty.def {
            assert_eq!(*inner_ref, pong_ref);
        } else {
            panic!("Expected Option");
        }
    } else {
        panic!("Expected Composite, got {:?}", ping_ty.def);
    }
}
