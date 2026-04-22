use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::{Range, RangeInclusive};
use core::time::Duration;
use sails_idl_ast::{PrimitiveType, TypeDecl, TypeDef};
use sails_type_registry::alloc;
use sails_type_registry::{MetaType, Registry};

#[test]
fn transparent_wrappers_inherit_inner_type_decl() {
    let mut registry = Registry::new();

    let inner = TypeDecl::Primitive(PrimitiveType::U32);

    assert_eq!(registry.decl_for::<Box<u32>>(), inner);
    assert_eq!(registry.decl_for::<Rc<u32>>(), inner);
    assert_eq!(registry.decl_for::<Arc<u32>>(), inner);
    assert_eq!(registry.decl_for::<Cow<'static, u32>>(), inner);
    assert_eq!(registry.decl_for::<&'static u32>(), inner);
    assert_eq!(registry.decl_for::<&'static mut u32>(), inner);

    assert!(registry.is_empty());
}

#[test]
fn phantom_data_is_registered_as_unit_struct() {
    let mut registry = Registry::new();

    let type_ref = registry.register_type::<PhantomData<u32>>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert_eq!(ty.name, "PhantomData");
    assert_eq!(ty.type_params.len(), 1);
    assert_eq!(ty.type_params[0].name, "T");

    let TypeDef::Struct(struct_def) = &ty.def else {
        panic!("expected struct, got {:?}", ty.def);
    };
    assert!(struct_def.fields.is_empty());
}

#[test]
fn duration_has_named_secs_and_nanos_fields() {
    let mut registry = Registry::new();

    let type_ref = registry.register_type::<Duration>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert_eq!(ty.name, "Duration");

    let TypeDef::Struct(struct_def) = &ty.def else {
        panic!("expected struct, got {:?}", ty.def);
    };
    assert_eq!(struct_def.fields.len(), 2);
    assert_eq!(struct_def.fields[0].name.as_deref(), Some("secs"));
    assert_eq!(
        struct_def.fields[0].type_decl,
        TypeDecl::Primitive(PrimitiveType::U64)
    );
    assert_eq!(struct_def.fields[1].name.as_deref(), Some("nanos"));
    assert_eq!(
        struct_def.fields[1].type_decl,
        TypeDecl::Primitive(PrimitiveType::U32)
    );
}

#[test]
fn ranges_store_abstract_start_and_end_fields() {
    let mut registry = Registry::new();

    let range_ref = registry.register_type::<Range<u32>>().unwrap();
    let range_ty = registry.get_type(range_ref).unwrap();

    assert_eq!(range_ty.name, "Range");
    let TypeDef::Struct(struct_def) = &range_ty.def else {
        panic!("expected struct");
    };
    assert_eq!(struct_def.fields.len(), 2);
    assert_eq!(struct_def.fields[0].name.as_deref(), Some("start"));
    assert_eq!(struct_def.fields[0].type_decl, TypeDecl::named("T".into()));
    assert_eq!(struct_def.fields[1].name.as_deref(), Some("end"));
    assert_eq!(struct_def.fields[1].type_decl, TypeDecl::named("T".into()));

    let inclusive_ref = registry.register_type::<RangeInclusive<u32>>().unwrap();
    let inclusive_ty = registry.get_type(inclusive_ref).unwrap();
    assert_eq!(inclusive_ty.name, "RangeInclusive");
}

#[test]
fn register_meta_type_accepts_nominal_handles() {
    let mut registry = Registry::new();

    let types = alloc::vec![
        MetaType::new::<Duration>(),
        MetaType::new::<Range<u32>>(),
        MetaType::new::<PhantomData<bool>>(),
    ];

    let refs: Vec<_> = types
        .into_iter()
        .map(|m| registry.register_meta_type(m))
        .collect();

    assert!(registry.is_type::<Duration>(refs[0]));
    assert!(registry.is_type::<Range<u32>>(refs[1]));
    assert!(registry.is_type::<PhantomData<bool>>(refs[2]));

    assert_eq!(
        registry.register_type::<Duration>(),
        Some(refs[0]),
        "IDs must stay stable across repeated registration"
    );
}

#[test]
fn register_type_returns_none_for_structural_declarations() {
    let mut registry = Registry::new();

    assert!(registry.register_type::<u32>().is_none());
    assert!(registry.register_type::<String>().is_none());
    assert!(registry.register_type::<()>().is_none());
    assert!(registry.register_type::<Vec<u8>>().is_none());
    assert!(registry.register_type::<Option<u32>>().is_none());
    assert!(registry.register_type::<(u32, bool)>().is_none());
    assert!(registry.is_empty());
}
