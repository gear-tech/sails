use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
use alloc::vec::Vec;
use sails_idl_ast::{StructDef, TypeDecl as AstTypeDecl, TypeDef};
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn test_container_types_in_template() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct AllContainers<T, E> {
        vec_field: Vec<T>,
        vec_deque_field: VecDeque<T>,
        btree_set_field: BTreeSet<T>,
        binary_heap_field: BinaryHeap<T>,
        option_field: Option<T>,
        result_field: Result<T, E>,
        btree_map_field: BTreeMap<T, E>,
        vec_option: Vec<Option<T>>,
        option_result: Option<Result<T, E>>,
    }

    let mut registry = Registry::new();

    // Register AllContainers<String, u32>
    let _ = registry.register_type::<AllContainers<String, u32>>();

    // Find the template
    let template = registry
        .named_types()
        .find(|t| t.name == "AllContainers")
        .expect("AllContainers template not found");

    // Template should have 2 type_params
    assert_eq!(template.type_params.len(), 2);
    assert_eq!(template.type_params[0].name, "T");
    assert_eq!(template.type_params[1].name, "E");

    // Check struct fields
    if let TypeDef::Struct(StructDef { fields }) = &template.def {
        assert_eq!(fields.len(), 9);

        // vec_field: Vec<T> → Slice { item: T_param }
        match &fields[0].type_decl {
            AstTypeDecl::Slice { item } => {
                assert_param(item, "T");
            }
            other => panic!("vec_field should be Slice, got {:?}", other),
        }

        // vec_deque_field: VecDeque<T> → Slice { item: T_param }
        match &fields[1].type_decl {
            AstTypeDecl::Slice { item } => {
                assert_param(item, "T");
            }
            other => panic!("vec_deque_field should be Slice, got {:?}", other),
        }

        // btree_set_field: BTreeSet<T> → Slice { item: T_param }
        match &fields[2].type_decl {
            AstTypeDecl::Slice { item } => {
                assert_param(item, "T");
            }
            other => panic!("btree_set_field should be Slice, got {:?}", other),
        }

        // binary_heap_field: BinaryHeap<T> → Slice { item: T_param }
        match &fields[3].type_decl {
            AstTypeDecl::Slice { item } => {
                assert_param(item, "T");
            }
            other => panic!("binary_heap_field should be Slice, got {:?}", other),
        }

        // option_field: Option<T> → Option(T_param)
        match &fields[4].type_decl {
            AstTypeDecl::Named { name, generics, .. } => {
                assert_eq!(name, "Option");
                assert_eq!(generics.len(), 1);
                assert_param(&generics[0], "T");
            }
            other => panic!("option_field should be Option, got {:?}", other),
        }

        // result_field: Result<T, E> → Result(T_param, E_param)
        match &fields[5].type_decl {
            AstTypeDecl::Named { name, generics, .. } => {
                assert_eq!(name, "Result");
                assert_eq!(generics.len(), 2);
                assert_param(&generics[0], "T");
                assert_param(&generics[1], "E");
            }
            other => panic!("result_field should be Result, got {:?}", other),
        }

        // btree_map_field: BTreeMap<T, E> → Slice { item: Tuple { types: [T_param, E_param] } }
        match &fields[6].type_decl {
            AstTypeDecl::Slice { item } => match item.as_ref() {
                AstTypeDecl::Tuple { types } => {
                    assert_eq!(types.len(), 2);
                    assert_param(&types[0], "T");
                    assert_param(&types[1], "E");
                }
                other => panic!("btree_map item should be Tuple, got {:?}", other),
            },
            other => panic!("btree_map_field should be Slice, got {:?}", other),
        }

        // vec_option: Vec<Option<T>> → Slice { item: Option(T_param) }
        match &fields[7].type_decl {
            AstTypeDecl::Slice { item } => match item.as_ref() {
                AstTypeDecl::Named { name, generics, .. } => {
                    assert_eq!(name, "Option");
                    assert_eq!(generics.len(), 1);
                    assert_param(&generics[0], "T");
                }
                other => panic!("vec_option item should be Option, got {:?}", other),
            },
            other => panic!("vec_option should be Slice, got {:?}", other),
        }

        // option_result: Option<Result<T, E>> → Option(Result(T_param, E_param))
        match &fields[8].type_decl {
            AstTypeDecl::Named { name, generics, .. } => {
                assert_eq!(name, "Option");
                assert_eq!(generics.len(), 1);
                match &generics[0] {
                    AstTypeDecl::Named { name, generics, .. } => {
                        assert_eq!(name, "Result");
                        assert_eq!(generics.len(), 2);
                        assert_param(&generics[0], "T");
                        assert_param(&generics[1], "E");
                    }
                    other => panic!("option_result inner should be Result, got {:?}", other),
                }
            }
            other => panic!("option_result should be Option, got {:?}", other),
        }
    } else {
        panic!("Expected Struct definition");
    }
}

fn assert_param(decl: &AstTypeDecl, expected_name: &str) {
    match decl {
        AstTypeDecl::Named { name, param, .. } => {
            assert_eq!(name, expected_name);
            assert_eq!(param, &Some(sails_idl_ast::NamedParam::Type));
        }
        other => panic!("Expected parameter {}, got {:?}", expected_name, other),
    }
}

#[test]
fn test_const_generics_in_template() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct WithConst<T, const N: usize> {
        array_field: [T; N],
        vec_field: Vec<T>,
        nested: Vec<[T; N]>,
    }

    let mut registry = Registry::new();

    // Register WithConst<String, 42>
    let _ = registry.register_type::<WithConst<String, 42>>();

    // Find the template
    let template = registry
        .named_types()
        .find(|t| t.name == "WithConst")
        .expect("WithConst template not found");

    // Template should have 2 type_params: T and N
    assert_eq!(template.type_params.len(), 2);
    assert_eq!(template.type_params[0].name, "T");
    assert_eq!(template.type_params[1].name, "N");

    // Check struct fields
    if let TypeDef::Struct(StructDef { fields }) = &template.def {
        assert_eq!(fields.len(), 3);

        // array_field: [T; N] → Array { item: T_param, len: 42 }
        match &fields[0].type_decl {
            AstTypeDecl::Array { item, len } => {
                assert_eq!(*len, 42);
                assert_param(item, "T");
            }
            other => panic!("array_field should be Array, got {:?}", other),
        }

        // vec_field: Vec<T> → Slice { item: T_param }
        match &fields[1].type_decl {
            AstTypeDecl::Slice { item } => {
                assert_param(item, "T");
            }
            other => panic!("vec_field should be Slice, got {:?}", other),
        }

        // nested: Vec<[T; N]> → Slice { item: Array { item: T_param, len: 42 } }
        match &fields[2].type_decl {
            AstTypeDecl::Slice { item } => match item.as_ref() {
                AstTypeDecl::Array { len, .. } => {
                    assert_eq!(*len, 42);
                }
                other => panic!("nested item should be Array, got {:?}", other),
            },
            other => panic!("nested should be Slice, got {:?}", other),
        }
    } else {
        panic!("Expected Struct definition");
    }
}
