extern crate alloc;

use alloc::{borrow::Cow, boxed::Box, rc::Rc, sync::Arc};
use core::{
    marker::PhantomData,
    ops::{Range, RangeInclusive},
    time::Duration,
};

use sails_type_registry::{Registry, ty::TypeDef};

#[test]
fn test_transparent_wrappers() {
    let mut registry = Registry::new();

    // The inner type
    let u32_ref = registry.register_type::<u32>();

    // Box
    let box_ref = registry.register_type::<Box<u32>>();
    assert_eq!(box_ref, u32_ref, "Box should be transparent");

    // Rc
    let rc_ref = registry.register_type::<Rc<u32>>();
    assert_eq!(rc_ref, u32_ref, "Rc should be transparent");

    // Arc
    let arc_ref = registry.register_type::<Arc<u32>>();
    assert_eq!(arc_ref, u32_ref, "Arc should be transparent");

    // Cow
    let cow_ref = registry.register_type::<Cow<'static, u32>>();
    assert_eq!(cow_ref, u32_ref, "Cow should be transparent");

    // References
    let ref_ref = registry.register_type::<&'static u32>();
    assert_eq!(ref_ref, u32_ref, "Reference should be transparent");

    let mut_ref_ref = registry.register_type::<&'static mut u32>();
    assert_eq!(mut_ref_ref, u32_ref, "Mut reference should be transparent");
}

#[test]
fn test_phantom_data() {
    let mut registry = Registry::new();

    let phantom_ref = registry.register_type::<PhantomData<u32>>();
    let phantom_ty = registry.get_type(phantom_ref).unwrap();

    // PhantomData is represented as an empty tuple with a specific path
    assert_eq!(phantom_ty.name, "PhantomData");
    if let TypeDef::Tuple(fields) = &phantom_ty.def {
        assert!(fields.is_empty());
    } else {
        panic!("Expected Tuple, got {:?}", phantom_ty.def);
    }
}

#[test]
fn test_duration() {
    let mut registry = Registry::new();

    let duration_ref = registry.register_type::<Duration>();
    let duration_ty = registry.get_type(duration_ref).unwrap();

    assert_eq!(duration_ty.name, "Duration");
    if let TypeDef::Composite(c) = &duration_ty.def {
        assert_eq!(c.fields.len(), 2);
        assert_eq!(c.fields[0].name.as_deref(), Some("secs"));
        assert_eq!(c.fields[1].name.as_deref(), Some("nanos"));
    } else {
        panic!("Expected Composite, got {:?}", duration_ty.def);
    }
}

#[test]
fn test_ranges() {
    let mut registry = Registry::new();

    let range_ref = registry.register_type::<Range<u32>>();
    let range_ty = registry.get_type(range_ref).unwrap();

    assert_eq!(range_ty.name, "Range");
    if let TypeDef::Composite(c) = &range_ty.def {
        assert_eq!(c.fields.len(), 2);
        assert_eq!(c.fields[0].name.as_deref(), Some("start"));
        assert_eq!(c.fields[1].name.as_deref(), Some("end"));
    } else {
        panic!("Expected Composite, got {:?}", range_ty.def);
    }

    let inclusive_ref = registry.register_type::<RangeInclusive<u32>>();
    let inclusive_ty = registry.get_type(inclusive_ref).unwrap();

    assert_eq!(inclusive_ty.name, "RangeInclusive");
}

#[test]
fn test_meta_type_registration() {
    use sails_type_registry::{MetaType, Registry};

    let mut registry = Registry::new();

    // Store different types in a single vector using MetaType
    let types = alloc::vec![
        MetaType::new::<u32>(),
        MetaType::new::<alloc::string::String>(),
        MetaType::new::<alloc::vec::Vec<u8>>(),
        MetaType::new::<Option<bool>>(),
    ];

    // Register all of them
    let refs: alloc::vec::Vec<_> = types.iter().map(|m| m.register(&mut registry)).collect();

    // Verify registrations
    assert!(registry.is_type::<u32>(refs[0]));
    assert!(registry.is_type::<alloc::string::String>(refs[1]));
    assert!(registry.is_type::<alloc::vec::Vec<u8>>(refs[2]));
    assert!(registry.is_type::<Option<bool>>(refs[3]));

    // Check that IDs are stable
    assert_eq!(registry.register_type::<u32>(), refs[0]);
}
