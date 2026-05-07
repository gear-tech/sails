use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
use core::num::NonZeroU32;
use sails_idl_ast::{PrimitiveType, TypeDecl};
use sails_type_registry::Registry;
use sails_type_registry::alloc;

fn u32_decl() -> TypeDecl {
    TypeDecl::Primitive(PrimitiveType::U32)
}

fn u8_decl() -> TypeDecl {
    TypeDecl::Primitive(PrimitiveType::U8)
}

fn bool_decl() -> TypeDecl {
    TypeDecl::Primitive(PrimitiveType::Bool)
}

fn string_decl() -> TypeDecl {
    TypeDecl::Primitive(PrimitiveType::String)
}

#[test]
fn sequence_collections_lower_to_slice_type_decl() {
    let mut registry = Registry::new();

    let expected = TypeDecl::Slice {
        item: Box::new(u32_decl()),
    };

    assert_eq!(registry.decl_for::<Vec<u32>>(), expected);
    assert_eq!(registry.decl_for::<VecDeque<u32>>(), expected);
    assert_eq!(registry.decl_for::<BTreeSet<u32>>(), expected);
    assert_eq!(registry.decl_for::<BinaryHeap<u32>>(), expected);
    assert_eq!(registry.decl_for::<[u32]>(), expected);

    assert!(
        registry.is_empty(),
        "sequence containers must not allocate registry slots"
    );
}

#[test]
fn fixed_arrays_lower_to_array_type_decl() {
    let mut registry = Registry::new();

    assert_eq!(
        registry.decl_for::<[u8; 32]>(),
        TypeDecl::Array {
            item: Box::new(u8_decl()),
            len: 32,
        }
    );
    assert!(registry.is_empty());
}

#[test]
fn btree_map_lowers_to_slice_of_tuple() {
    let mut registry = Registry::new();

    let decl = registry.decl_for::<BTreeMap<NonZeroU32, bool>>();
    let (nz_name, nz_generics) = match decl {
        TypeDecl::Slice { item } => match *item {
            TypeDecl::Tuple { types } => {
                assert_eq!(types.len(), 2);
                assert_eq!(types[1], bool_decl());
                match &types[0] {
                    TypeDecl::Named { name, generics } => (name.clone(), generics.clone()),
                    other => panic!("expected NonZeroU32 as Named, got {other:?}"),
                }
            }
            other => panic!("expected tuple element, got {other:?}"),
        },
        other => panic!("expected Slice<Tuple>, got {other:?}"),
    };
    assert_eq!(nz_name, "NonZeroU32");
    assert!(nz_generics.is_empty());
}

#[test]
fn option_and_result_lower_to_named_with_generics() {
    let mut registry = Registry::new();

    assert_eq!(
        registry.decl_for::<Option<u32>>(),
        TypeDecl::Named {
            name: "Option".into(),
            generics: alloc::vec![u32_decl()],
        }
    );

    assert_eq!(
        registry.decl_for::<Result<u32, bool>>(),
        TypeDecl::Named {
            name: "Result".into(),
            generics: alloc::vec![u32_decl(), bool_decl()],
        }
    );

    assert!(registry.is_empty());
}

#[test]
fn tuples_lower_to_tuple_type_decl() {
    let mut registry = Registry::new();

    assert_eq!(
        registry.decl_for::<()>(),
        TypeDecl::Primitive(PrimitiveType::Void)
    );

    assert_eq!(
        registry.decl_for::<(u32, bool)>(),
        TypeDecl::Tuple {
            types: alloc::vec![u32_decl(), bool_decl()],
        }
    );

    assert_eq!(
        registry.decl_for::<(u64,)>(),
        TypeDecl::Tuple {
            types: alloc::vec![TypeDecl::Primitive(PrimitiveType::U64)],
        }
    );

    assert_eq!(
        registry.decl_for::<(u8, u16, u32)>(),
        TypeDecl::Tuple {
            types: alloc::vec![
                u8_decl(),
                TypeDecl::Primitive(PrimitiveType::U16),
                u32_decl(),
            ],
        }
    );

    assert_eq!(
        registry.decl_for::<((u32, bool), u64)>(),
        TypeDecl::Tuple {
            types: alloc::vec![
                TypeDecl::Tuple {
                    types: alloc::vec![u32_decl(), bool_decl()],
                },
                TypeDecl::Primitive(PrimitiveType::U64),
            ],
        }
    );

    assert!(registry.is_empty());
}

#[test]
fn complex_tuples_lower_recursively() {
    type ComplexTuple = (
        u32,
        Vec<u8>,
        Option<bool>,
        BTreeMap<u32, String>,
        (u64, i64),
    );

    let mut registry = Registry::new();
    let decl = registry.decl_for::<ComplexTuple>();

    let expected = TypeDecl::Tuple {
        types: alloc::vec![
            u32_decl(),
            TypeDecl::Slice {
                item: Box::new(u8_decl()),
            },
            TypeDecl::Named {
                name: "Option".into(),
                generics: alloc::vec![bool_decl()],
            },
            TypeDecl::Slice {
                item: Box::new(TypeDecl::Tuple {
                    types: alloc::vec![u32_decl(), string_decl()],
                }),
            },
            TypeDecl::Tuple {
                types: alloc::vec![
                    TypeDecl::Primitive(PrimitiveType::U64),
                    TypeDecl::Primitive(PrimitiveType::I64),
                ],
            },
        ],
    };

    assert_eq!(decl, expected);
    assert!(registry.is_empty());
}
