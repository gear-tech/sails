use alloc::string::{String, ToString};
use alloc::vec;
use sails_idl_ast::TypeDef;
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn derive_captures_docs_on_types_fields_variants() {
    /// This is a test struct.
    /// It has multiple lines of docs.
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct DocStruct {
        /// The unique identifier.
        _id: u32,
        /// The name of the entity.
        _name: String,
    }

    /// This is a test enum.
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum DocEnum {
        /// First variant.
        _First,
        /// Second variant with fields.
        _Second {
            /// A nested field.
            _f: bool,
        },
    }

    let mut registry = Registry::new();

    let struct_ref = registry.register_type::<DocStruct>().unwrap();
    let struct_ty = registry.get_type(struct_ref).unwrap();

    assert_eq!(
        struct_ty.docs,
        vec![
            "This is a test struct.".to_string(),
            "It has multiple lines of docs.".to_string(),
        ]
    );

    let TypeDef::Struct(struct_def) = &struct_ty.def else {
        panic!("expected struct");
    };
    assert_eq!(
        struct_def.fields[0].docs,
        vec!["The unique identifier.".to_string()]
    );
    assert_eq!(
        struct_def.fields[1].docs,
        vec!["The name of the entity.".to_string()]
    );

    let enum_ref = registry.register_type::<DocEnum>().unwrap();
    let enum_ty = registry.get_type(enum_ref).unwrap();

    assert_eq!(enum_ty.docs, vec!["This is a test enum.".to_string()]);

    let TypeDef::Enum(enum_def) = &enum_ty.def else {
        panic!("expected enum");
    };
    assert_eq!(
        enum_def.variants[0].docs,
        vec!["First variant.".to_string()]
    );
    assert_eq!(
        enum_def.variants[1].docs,
        vec!["Second variant with fields.".to_string()]
    );
    assert_eq!(
        enum_def.variants[1].def.fields[0].docs,
        vec!["A nested field.".to_string()]
    );
}
