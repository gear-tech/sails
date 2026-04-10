use sails_type_registry::ty::{Annotation, TypeDef};
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn test_struct_annotation() {
    #[derive(TypeInfo)]
    #[type_info(indexed)]
    struct Indexed {
        _x: u32,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<Indexed>();
    let ty = registry.get_type(ty_ref).unwrap();

    assert_eq!(ty.annotations, [Annotation::new("indexed")]);
}

#[test]
fn test_multiple_annotations_on_struct() {
    #[derive(TypeInfo)]
    #[type_info(compressed)]
    #[type_info(versioned)]
    struct MultiAnnotated {
        _data: u32,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<MultiAnnotated>();
    let ty = registry.get_type(ty_ref).unwrap();

    assert_eq!(
        ty.annotations,
        [Annotation::new("compressed"), Annotation::new("versioned")]
    );
}

#[test]
fn test_enum_annotation() {
    #[derive(TypeInfo)]
    #[type_info(sealed)]
    enum Sealed {
        _A,
        _B,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<Sealed>();
    let ty = registry.get_type(ty_ref).unwrap();

    assert_eq!(ty.annotations, [Annotation::new("sealed")]);
}

#[test]
fn test_variant_annotation() {
    #[derive(TypeInfo)]
    enum Tagged {
        #[type_info(deprecated)]
        _Old,
        _New,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<Tagged>();
    let ty = registry.get_type(ty_ref).unwrap();

    if let TypeDef::Variant(v) = &ty.def {
        assert_eq!(v.variants[0].annotations, [Annotation::new("deprecated")]);
        assert!(v.variants[1].annotations.is_empty());
    } else {
        panic!("Expected Variant, got {:?}", ty.def);
    }
}

#[test]
fn test_field_annotation() {
    #[derive(TypeInfo)]
    struct WithFieldAnnotation {
        #[type_info(sensitive)]
        _secret: u64,
        _public: u32,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<WithFieldAnnotation>();
    let ty = registry.get_type(ty_ref).unwrap();

    if let TypeDef::Composite(c) = &ty.def {
        assert_eq!(c.fields[0].annotations, [Annotation::new("sensitive")]);
        assert!(c.fields[1].annotations.is_empty());
    } else {
        panic!("Expected Composite, got {:?}", ty.def);
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

    assert!(ty.annotations.is_empty());
}

#[test]
fn test_annotation_with_value() {
    #[derive(TypeInfo)]
    #[type_info(version = "2")]
    struct Versioned {
        _x: u32,
    }

    let mut registry = Registry::new();
    let ty_ref = registry.register_type::<Versioned>();
    let ty = registry.get_type(ty_ref).unwrap();

    assert_eq!(
        ty.annotations,
        [Annotation {
            name: "version".into(),
            value: Some("2".into()),
        }]
    );
}
