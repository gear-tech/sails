extern crate alloc;

use alloc::string::String;
use sails_type_registry::ty::TypeDef;
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn test_documentation_collection() {
    /// This is a test struct.
    /// It has multiple lines of docs.
    #[derive(TypeInfo)]
    struct DocStruct {
        /// The unique identifier.
        _id: u32,
        /// The name of the entity.
        _name: String,
    }

    /// This is a test enum.
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

    // Check struct docs
    let struct_ref = registry.register_type::<DocStruct>();
    let struct_ty = registry.get_type(struct_ref).unwrap();

    assert_eq!(
        struct_ty.docs,
        vec![
            "This is a test struct.".to_string(),
            "It has multiple lines of docs.".to_string()
        ]
    );

    if let TypeDef::Composite(c) = &struct_ty.def {
        assert_eq!(c.fields[0].docs, vec!["The unique identifier.".to_string()]);
        assert_eq!(
            c.fields[1].docs,
            vec!["The name of the entity.".to_string()]
        );
    }

    // Check enum docs
    let enum_ref = registry.register_type::<DocEnum>();
    let enum_ty = registry.get_type(enum_ref).unwrap();

    assert_eq!(enum_ty.docs, vec!["This is a test enum.".to_string()]);

    if let TypeDef::Variant(v) = &enum_ty.def {
        assert_eq!(v.variants[0].docs, vec!["First variant.".to_string()]);
        assert_eq!(
            v.variants[1].docs,
            vec!["Second variant with fields.".to_string()]
        );
        assert_eq!(
            v.variants[1].fields[0].docs,
            vec!["A nested field.".to_string()]
        );
    }
}
