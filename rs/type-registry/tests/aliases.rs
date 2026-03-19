extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use sails_type_registry::ty::{FieldType, TypeDef, TypeDefinitionKind};
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn test_complex_aliases_expansion() {
    #[allow(dead_code)]
    type MyResult<T> = Result<T, String>;

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct MyData<T> {
        res: MyResult<T>,
        opt: Option<T>,
        _list: Vec<T>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Wrapper {
        data: MyData<u32>,
    }

    let mut registry = Registry::new();
    let _u32_id = registry.register_type::<u32>();
    let string_id = registry.register_type::<String>();

    let wrapper_ref = registry.register_type::<Wrapper>();
    let wrapper_ty = registry
        .get_type(wrapper_ref)
        .expect("Wrapper type not found");

    if let TypeDef::Definition(def) = &wrapper_ty.def
        && let TypeDefinitionKind::Composite(comp) = &def.kind
    {
        let data_field = &comp.fields[0];
        if let FieldType::Id(data_id) = data_field.ty {
            let data_ty = registry
                .get_type(data_id)
                .expect("MyData<u32> type not found");

            if let TypeDef::Definition(data_def) = &data_ty.def
                && let TypeDefinitionKind::Composite(data_comp) = &data_def.kind
            {
                // Check MyResult<u32> (which is Result<u32, String>)
                let res_field = &data_comp.fields[0];
                // MyResult<T> is now Id, and its expansion in MyData<u32> should be Result<u32, String>
                match &res_field.ty {
                    FieldType::Parameterized { id, args } => {
                        let res_ty = registry.get_type(*id).expect("Result type not found");
                        assert!(matches!(res_ty.def, TypeDef::Result { .. }));
                        assert_eq!(
                            args.len(),
                            2,
                            "Result alias should be expanded to 2 arguments"
                        );
                        assert_eq!(args[0], FieldType::Parameter("T".to_string()));
                        assert_eq!(args[1], FieldType::Id(string_id));
                    }
                    _ => panic!(
                        "Expected Parameterized field type for res, got {:?}",
                        res_field.ty
                    ),
                }

                // Check Option<u32>
                let opt_field = &data_comp.fields[1];
                match &opt_field.ty {
                    FieldType::Parameterized { id, args } => {
                        let opt_ty = registry.get_type(*id).expect("Option type not found");
                        assert!(matches!(opt_ty.def, TypeDef::Option(_)));
                        assert_eq!(args.len(), 1);
                        assert_eq!(args[0], FieldType::Parameter("T".to_string()));
                    }
                    _ => panic!("Expected Parameterized field type for opt"),
                }
            }
        }
    }
}

#[test]
fn test_nested_aliases() {
    #[allow(dead_code)]
    type Inner<T> = (T, bool);
    #[allow(dead_code)]
    type Outer<T> = Vec<Inner<T>>;

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Nested {
        field: Outer<u32>,
    }

    let mut registry = Registry::new();
    let u32_id = registry.register_type::<u32>();
    let bool_id = registry.register_type::<bool>();

    let nested_ref = registry.register_type::<Nested>();
    let nested_ty = registry.get_type(nested_ref).unwrap();

    if let TypeDef::Definition(def) = &nested_ty.def
        && let TypeDefinitionKind::Composite(comp) = &def.kind
    {
        let field = &comp.fields[0];
        // Outer<u32> is parameterized, Inner<u32> inside should be expanded to Tuple
        match &field.ty {
            FieldType::Parameterized { id, args } => {
                let vec_ty = registry.get_type(*id).unwrap();
                assert!(matches!(vec_ty.def, TypeDef::Sequence(_)));
                assert_eq!(args.len(), 1);

                match &args[0] {
                    FieldType::Tuple { elems, .. } => {
                        assert_eq!(elems.len(), 2);
                        assert_eq!(elems[0], FieldType::Id(u32_id));
                        assert_eq!(elems[1], FieldType::Id(bool_id));
                    }
                    _ => panic!(
                        "Expected Tuple field type for Inner<u32>, got {:?}",
                        args[0]
                    ),
                }
            }
            _ => panic!(
                "Expected Parameterized field type for field, got {:?}",
                field.ty
            ),
        }
    }
}
