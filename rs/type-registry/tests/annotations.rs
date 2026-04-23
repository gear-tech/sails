use alloc::string::ToString;
use sails_idl_ast::TypeDef;
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn derive_preserves_struct_annotation() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    #[annotate(indexed)]
    struct Indexed {
        _x: u32,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<Indexed>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert_eq!(ty.annotations, [("indexed".to_string(), None)]);
}

#[test]
fn derive_preserves_multiple_struct_annotations() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    #[annotate(compressed)]
    #[annotate(versioned)]
    struct MultiAnnotated {
        _data: u32,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<MultiAnnotated>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert_eq!(
        ty.annotations,
        [
            ("compressed".to_string(), None),
            ("versioned".to_string(), None)
        ]
    );
}

#[test]
fn derive_preserves_enum_annotation() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    #[annotate(sealed)]
    enum Sealed {
        _A,
        _B,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<Sealed>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert_eq!(ty.annotations, [("sealed".to_string(), None)]);
}

#[test]
fn derive_preserves_variant_annotation() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum Tagged {
        #[annotate(deprecated)]
        _Old,
        _New,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<Tagged>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    let TypeDef::Enum(enum_def) = &ty.def else {
        panic!("Expected Enum, got {:?}", ty.def);
    };
    assert_eq!(
        enum_def.variants[0].annotations,
        [("deprecated".to_string(), None)]
    );
    assert!(enum_def.variants[1].annotations.is_empty());
}

#[test]
fn derive_preserves_field_annotation() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct WithFieldAnnotation {
        #[annotate(sensitive)]
        _secret: u64,
        _public: u32,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<WithFieldAnnotation>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    let TypeDef::Struct(struct_def) = &ty.def else {
        panic!("Expected Struct, got {:?}", ty.def);
    };
    assert_eq!(
        struct_def.fields[0].annotations,
        [("sensitive".to_string(), None)]
    );
    assert!(struct_def.fields[1].annotations.is_empty());
}

#[test]
fn derive_leaves_no_default_annotations() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Plain {
        _x: u32,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<Plain>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert!(ty.annotations.is_empty());
}

#[test]
fn derive_preserves_annotation_with_value() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    #[annotate(version = "2")]
    struct Versioned {
        _x: u32,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<Versioned>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert_eq!(
        ty.annotations,
        [("version".to_string(), Some("2".to_string()))]
    );
}
