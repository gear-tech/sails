use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
use alloc::vec::Vec;
use core::num::NonZeroU32;
use sails_type_registry::alloc;

use sails_type_registry::{Registry, ty::TypeDef};

#[test]
fn test_sequences() {
    let mut registry = Registry::new();

    let u32_ref = registry.register_type::<u32>();

    // Test Vec
    let vec_ref = registry.register_type::<Vec<u32>>();
    let vec_ty = registry.get_type(vec_ref).unwrap();
    assert_eq!(vec_ty.def, TypeDef::Sequence(u32_ref));

    // Test VecDeque
    let deque_ref = registry.register_type::<VecDeque<u32>>();
    let deque_ty = registry.get_type(deque_ref).unwrap();
    assert_eq!(deque_ty.def, TypeDef::Sequence(u32_ref));

    // Test BTreeSet
    let set_ref = registry.register_type::<BTreeSet<u32>>();
    let set_ty = registry.get_type(set_ref).unwrap();
    assert_eq!(set_ty.def, TypeDef::Sequence(u32_ref));

    // Test BinaryHeap
    let heap_ref = registry.register_type::<BinaryHeap<u32>>();
    let heap_ty = registry.get_type(heap_ref).unwrap();
    assert_eq!(heap_ty.def, TypeDef::Sequence(u32_ref));
}

#[test]
fn test_arrays_and_slices() {
    let mut registry = Registry::new();
    let u8_ref = registry.register_type::<u8>();

    // Test slice `[T]`
    let slice_ref = registry.register_type::<[u8]>();
    let slice_ty = registry.get_type(slice_ref).unwrap();
    assert_eq!(slice_ty.def, TypeDef::Sequence(u8_ref));

    // Test fixed size array `[T; N]`
    let array_ref = registry.register_type::<[u8; 32]>();
    let array_ty = registry.get_type(array_ref).unwrap();
    assert_eq!(
        array_ty.def,
        TypeDef::Array {
            len: 32,
            type_param: u8_ref
        }
    );
}

#[test]
fn test_map() {
    let mut registry = Registry::new();

    let key_ref = registry.register_type::<NonZeroU32>();
    let val_ref = registry.register_type::<bool>();

    let map_ref = registry.register_type::<BTreeMap<NonZeroU32, bool>>();
    let map_ty = registry.get_type(map_ref).unwrap();

    assert_eq!(
        map_ty.def,
        TypeDef::Map {
            key: key_ref,
            value: val_ref
        }
    );
}

#[test]
fn test_option_and_result() {
    let mut registry = Registry::new();

    let u32_ref = registry.register_type::<u32>();
    let bool_ref = registry.register_type::<bool>();

    let opt_ref = registry.register_type::<Option<u32>>();
    let opt_ty = registry.get_type(opt_ref).unwrap();
    assert_eq!(opt_ty.def, TypeDef::Option(u32_ref));

    let res_ref = registry.register_type::<Result<u32, bool>>();
    let res_ty = registry.get_type(res_ref).unwrap();
    assert_eq!(
        res_ty.def,
        TypeDef::Result {
            ok: u32_ref,
            err: bool_ref
        }
    );
}

#[test]
fn test_tuples() {
    let mut registry = Registry::new();

    let u32_ref = registry.register_type::<u32>();
    let bool_ref = registry.register_type::<bool>();

    // Empty tuple (unit)
    let unit_ref = registry.register_type::<()>();
    let unit_ty = registry.get_type(unit_ref).unwrap();
    assert_eq!(unit_ty.def, TypeDef::Tuple(Vec::new()));

    // Regular tuple
    let tuple_ref = registry.register_type::<(u32, bool)>();
    let tuple_ty = registry.get_type(tuple_ref).unwrap();
    assert_eq!(tuple_ty.def, TypeDef::Tuple(alloc::vec![u32_ref, bool_ref]));

    // 1-element tuple
    let t1_ref = registry.register_type::<(u64,)>();
    let t1_ty = registry.get_type(t1_ref).unwrap();
    if let TypeDef::Tuple(fields) = &t1_ty.def {
        assert_eq!(fields.len(), 1);
        assert!(registry.is_type::<u64>(fields[0]));
    } else {
        panic!("Expected 1-tuple");
    }

    // 3-element tuple
    let t3_ref = registry.register_type::<(u8, u16, u32)>();
    let t3_ty = registry.get_type(t3_ref).unwrap();
    if let TypeDef::Tuple(fields) = &t3_ty.def {
        assert_eq!(fields.len(), 3);
        assert!(registry.is_type::<u8>(fields[0]));
        assert!(registry.is_type::<u16>(fields[1]));
        assert!(registry.is_type::<u32>(fields[2]));
    } else {
        panic!("Expected 3-tuple");
    }

    // Nested tuple: ((u32, bool), u64)
    let nested_ref = registry.register_type::<((u32, bool), u64)>();
    let nested_ty = registry.get_type(nested_ref).unwrap();
    if let TypeDef::Tuple(fields) = &nested_ty.def {
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0], tuple_ref);
        assert!(registry.is_type::<u64>(fields[1]));
    } else {
        panic!("Expected nested tuple");
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
    let tuple_ty = registry.get_type(tuple_ref).expect("Tuple not found");

    if let TypeDef::Tuple(fields) = &tuple_ty.def {
        assert_eq!(fields.len(), 5, "Tuple should have 5 elements");

        // Element 0: u32
        assert!(
            registry.is_type::<u32>(fields[0]),
            "Element 0 should be u32"
        );

        // Element 1: Vec<u8>
        assert!(
            registry.is_type::<alloc::vec::Vec<u8>>(fields[1]),
            "Element 1 should be Vec<u8>"
        );

        // Element 2: Option<bool>
        assert!(
            registry.is_type::<Option<bool>>(fields[2]),
            "Element 2 should be Option<bool>"
        );

        // Element 3: BTreeMap<u32, String>
        assert!(
            registry.is_type::<alloc::collections::BTreeMap<u32, alloc::string::String>>(fields[3]),
            "Element 3 should be BTreeMap<u32, String>"
        );

        // Element 4: (u64, i64)
        assert!(
            registry.is_type::<(u64, i64)>(fields[4]),
            "Element 4 should be (u64, i64)"
        );

        // Deep check Element 4 (Nested Tuple)
        let inner_tuple_ty = registry.get_type(fields[4]).unwrap();
        if let TypeDef::Tuple(inner_fields) = &inner_tuple_ty.def {
            assert_eq!(inner_fields.len(), 2);
            assert!(registry.is_type::<u64>(inner_fields[0]));
            assert!(registry.is_type::<i64>(inner_fields[1]));
        } else {
            panic!("Element 4 should be a tuple");
        }
    } else {
        panic!("Expected TypeDef::Tuple");
    }
}
