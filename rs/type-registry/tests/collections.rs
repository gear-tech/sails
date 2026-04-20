use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
use alloc::vec::Vec;
use core::num::NonZeroU32;
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeDecl};

#[test]
fn test_sequences() {
    let mut registry = Registry::new();

    // Test Vec
    let vec_ref = registry.register_type::<Vec<u32>>();
    let vec_decl = registry.get_type_decl(vec_ref).unwrap();
    assert!(matches!(vec_decl, TypeDecl::Slice { .. }));

    // Test VecDeque
    let deque_ref = registry.register_type::<VecDeque<u32>>();
    let deque_decl = registry.get_type_decl(deque_ref).unwrap();
    assert!(matches!(deque_decl, TypeDecl::Slice { .. }));

    // Test BTreeSet
    let set_ref = registry.register_type::<BTreeSet<u32>>();
    let set_decl = registry.get_type_decl(set_ref).unwrap();
    assert!(matches!(set_decl, TypeDecl::Slice { .. }));

    // Test BinaryHeap
    let heap_ref = registry.register_type::<BinaryHeap<u32>>();
    let heap_decl = registry.get_type_decl(heap_ref).unwrap();
    assert!(matches!(heap_decl, TypeDecl::Slice { .. }));
}

#[test]
fn test_arrays_and_slices() {
    let mut registry = Registry::new();
    let _u8_ref = registry.register_type::<u8>();

    // Test slice `[T]`
    let slice_ref = registry.register_type::<[u8]>();
    let slice_decl = registry.get_type_decl(slice_ref).unwrap();
    assert!(matches!(slice_decl, TypeDecl::Slice { .. }));

    // Test fixed size array `[T; N]`
    let array_ref = registry.register_type::<[u8; 32]>();
    let array_decl = registry.get_type_decl(array_ref).unwrap();
    match array_decl {
        TypeDecl::Array { len, .. } => assert_eq!(*len, 32),
        _ => panic!("Expected Array type"),
    }
}

#[test]
fn test_map() {
    let mut registry = Registry::new();

    let _key_ref = registry.register_type::<NonZeroU32>();
    let _val_ref = registry.register_type::<bool>();

    let map_ref = registry.register_type::<BTreeMap<NonZeroU32, bool>>();
    let map_decl = registry.get_type_decl(map_ref).unwrap();

    // BTreeMap is represented as Slice of Tuples (key-value pairs)
    match map_decl {
        TypeDecl::Slice { item } => match item.as_ref() {
            TypeDecl::Tuple { types } => {
                assert_eq!(
                    types.len(),
                    2,
                    "BTreeMap tuple should have 2 elements (key, value)"
                );
            }
            _ => panic!("Expected Tuple inside Slice for BTreeMap"),
        },
        _ => panic!("Expected Slice type for BTreeMap"),
    }
}

#[test]
fn test_option_and_result() {
    let mut registry = Registry::new();

    let _u32_ref = registry.register_type::<u32>();
    let _bool_ref = registry.register_type::<bool>();

    let opt_ref = registry.register_type::<Option<u32>>();
    let opt_decl = registry.get_type_decl(opt_ref).unwrap();
    match opt_decl {
        TypeDecl::Named { name, generics, .. } => {
            assert_eq!(name, "Option");
            assert_eq!(generics.len(), 1);
        }
        _ => panic!("Expected Named type for Option"),
    }

    let res_ref = registry.register_type::<Result<u32, bool>>();
    let res_decl = registry.get_type_decl(res_ref).unwrap();
    match res_decl {
        TypeDecl::Named { name, generics, .. } => {
            assert_eq!(name, "Result");
            assert_eq!(generics.len(), 2);
        }
        _ => panic!("Expected Named type for Result"),
    }
}

#[test]
fn test_tuples() {
    let mut registry = Registry::new();

    let _u32_ref = registry.register_type::<u32>();
    let _bool_ref = registry.register_type::<bool>();

    // Empty tuple (unit) - represented as Primitive(Void)
    let unit_ref = registry.register_type::<()>();
    let unit_decl = registry.get_type_decl(unit_ref).unwrap();
    match unit_decl {
        TypeDecl::Primitive(p) => assert_eq!(*p, sails_idl_ast::PrimitiveType::Void),
        _ => panic!("Expected Primitive(Void) type for unit"),
    }

    // Regular tuple
    let tuple_ref = registry.register_type::<(u32, bool)>();
    let tuple_decl = registry.get_type_decl(tuple_ref).unwrap();
    match tuple_decl {
        TypeDecl::Tuple { types } => {
            assert_eq!(types.len(), 2);
            assert!(registry.is_type_decl::<u32>(&types[0]));
            assert!(registry.is_type_decl::<bool>(&types[1]));
        }
        _ => panic!("Expected Tuple type"),
    }

    // 1-element tuple
    let t1_ref = registry.register_type::<(u64,)>();
    let t1_decl = registry.get_type_decl(t1_ref).unwrap();
    match t1_decl {
        TypeDecl::Tuple { types } => assert_eq!(types.len(), 1),
        _ => panic!("Expected Tuple type"),
    }

    // 3-element tuple
    let t3_ref = registry.register_type::<(u8, u16, u32)>();
    let t3_decl = registry.get_type_decl(t3_ref).unwrap();
    match t3_decl {
        TypeDecl::Tuple { types } => {
            assert_eq!(types.len(), 3);
            assert!(registry.is_type_decl::<u8>(&types[0]));
            assert!(registry.is_type_decl::<u16>(&types[1]));
            assert!(registry.is_type_decl::<u32>(&types[2]));
        }
        _ => panic!("Expected Tuple type"),
    }

    // Nested tuple: ((u32, bool), u64)
    let nested_ref = registry.register_type::<((u32, bool), u64)>();
    let nested_decl = registry.get_type_decl(nested_ref).unwrap();
    match nested_decl {
        TypeDecl::Tuple { types } => {
            assert_eq!(types.len(), 2);
            if let TypeDecl::Tuple { types: inner } = &types[0] {
                assert_eq!(inner.len(), 2);
            } else {
                panic!("Expected nested Tuple type");
            }
        }
        _ => panic!("Expected Tuple type"),
    }
}

#[test]
fn test_complex_tuples() {
    let mut registry = Registry::new();

    type ComplexTuple = (
        u32,
        alloc::vec::Vec<u8>,
        Option<bool>,
        alloc::collections::BTreeMap<u32, alloc::string::String>,
        (u64, i64),
    );

    let tuple_ref = registry.register_type::<ComplexTuple>();
    let tuple_decl = registry.get_type_decl(tuple_ref).expect("Tuple not found");

    if let TypeDecl::Tuple { types } = tuple_decl {
        assert_eq!(types.len(), 5, "Tuple should have 5 elements");

        // Element 0: u32
        assert!(
            registry.is_type_decl::<u32>(&types[0]),
            "Element 0 should be u32"
        );

        // Element 1: Vec<u8>
        assert!(
            registry.is_type_decl::<alloc::vec::Vec<u8>>(&types[1]),
            "Element 1 should be Vec<u8>"
        );

        // Element 2: Option<bool>
        assert!(
            registry.is_type_decl::<Option<bool>>(&types[2]),
            "Element 2 should be Option<bool>"
        );

        // Element 3: BTreeMap<u32, String>
        assert!(
            registry.is_type_decl::<alloc::collections::BTreeMap<u32, alloc::string::String>>(
                &types[3]
            ),
            "Element 3 should be BTreeMap<u32, String>"
        );

        // Element 4: (u64, i64)
        assert!(
            registry.is_type_decl::<(u64, i64)>(&types[4]),
            "Element 4 should be (u64, i64)"
        );

        // Deep check Element 4 (Nested Tuple)
        if let TypeDecl::Tuple { types: inner_types } = &types[4] {
            assert_eq!(inner_types.len(), 2);
            assert!(registry.is_type_decl::<u64>(&inner_types[0]));
            assert!(registry.is_type_decl::<i64>(&inner_types[1]));
        } else {
            panic!("Element 4 should be a tuple");
        }
    } else {
        panic!("Expected TypeDecl::Tuple");
    }
}
