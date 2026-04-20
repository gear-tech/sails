use sails_idl_ast::{StructDef, TypeDef};
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn test_struct_annotation() {
    #[derive(TypeInfo)]
    #[annotate(indexed)]
    struct Indexed {
        _x: u32,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<Indexed>();
    let ty = registry.get_type(ty_ref).unwrap();

    // module_path adds "path" annotation, plus our "indexed"
    assert!(ty.annotations.iter().any(|(n, _)| n == "path"));
    assert!(ty.annotations.iter().any(|(n, _)| n == "indexed"));
}

#[test]
fn test_multiple_annotations_on_struct() {
    #[derive(TypeInfo)]
    #[annotate(compressed)]
    #[annotate(versioned)]
    struct MultiAnnotated {
        _data: u32,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<MultiAnnotated>();
    let ty = registry.get_type(ty_ref).unwrap();

    // module_path adds "path" annotation
    assert!(ty.annotations.iter().any(|(n, _)| n == "path"));
    assert!(ty.annotations.iter().any(|(n, _)| n == "compressed"));
    assert!(ty.annotations.iter().any(|(n, _)| n == "versioned"));
}

#[test]
fn test_enum_annotation() {
    #[derive(TypeInfo)]
    #[annotate(sealed)]
    enum Sealed {
        _A,
        _B,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<Sealed>();
    let ty = registry.get_type(ty_ref).unwrap();

    // module_path adds "path" annotation
    assert!(ty.annotations.iter().any(|(n, _)| n == "path"));
    assert!(ty.annotations.iter().any(|(n, _)| n == "sealed"));
}

#[test]
fn test_variant_annotation() {
    #[derive(TypeInfo)]
    enum Tagged {
        #[annotate(deprecated)]
        _Old,
        _New,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<Tagged>();
    let ty = registry.get_type(ty_ref).unwrap();

    if let TypeDef::Enum(e) = &ty.def {
        assert_eq!(
            e.variants[0].annotations,
            [("deprecated".to_string(), None)]
        );
        assert!(e.variants[1].annotations.is_empty());
    } else {
        panic!("Expected Enum, got {:?}", ty.def);
    }
}

#[test]
fn test_field_annotation() {
    #[derive(TypeInfo)]
    struct WithFieldAnnotation {
        #[annotate(sensitive)]
        _secret: u64,
        _public: u32,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<WithFieldAnnotation>();
    let ty = registry.get_type(ty_ref).unwrap();

    if let TypeDef::Struct(StructDef { fields }) = &ty.def {
        assert_eq!(fields[0].annotations, [("sensitive".to_string(), None)]);
        assert!(fields[1].annotations.is_empty());
    } else {
        panic!("Expected Struct, got {:?}", ty.def);
    }
}

#[test]
fn test_no_annotations_by_default() {
    #[derive(TypeInfo)]
    struct Plain {
        _x: u32,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<Plain>();
    let ty = registry.get_type(ty_ref).unwrap();

    // module_path adds "path" annotation, but no user annotations
    assert!(ty.annotations.iter().any(|(n, _)| n == "path"));
    // No other annotations besides path
    assert_eq!(ty.annotations.len(), 1);
}

#[test]
fn test_annotation_with_value() {
    #[derive(TypeInfo)]
    #[annotate(version = "2")]
    struct Versioned {
        _x: u32,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<Versioned>();
    let ty = registry.get_type(ty_ref).unwrap();

    // Check for version annotation with value
    assert!(
        ty.annotations
            .iter()
            .any(|(n, v)| n == "version" && v.as_deref() == Some("2"))
    );
}
