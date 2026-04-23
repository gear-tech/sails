use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use core::marker::PhantomData;
use sails_idl_ast::{TypeDecl, TypeDef};
use sails_type_registry::alloc;
use sails_type_registry::{Registry, TypeInfo};

#[test]
fn type_only_generics_share_one_stored_def() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Point<T> {
        x: T,
        y: T,
    }

    let mut registry = Registry::new();
    let decl_u32 = <Point<u32> as TypeInfo>::type_decl(&mut registry);
    let decl_string = <Point<String> as TypeInfo>::type_decl(&mut registry);

    assert_eq!(decl_u32.to_string(), "Point<u32>");
    assert_eq!(decl_string.to_string(), "Point<String>");

    let ref_u32 = registry.get_registered::<Point<u32>>().unwrap().type_ref;
    let ref_string = registry.get_registered::<Point<String>>().unwrap().type_ref;
    assert_eq!(ref_u32, ref_string);

    let ty = registry.get_type(ref_u32).unwrap();
    assert_eq!(ty.name, "Point");
    assert_eq!(ty.type_params.len(), 1);
    assert_eq!(ty.type_params[0].name, "T");
    assert!(ty.type_params[0].ty.is_none());
}

#[test]
fn mixed_generics_share_one_const_suffixed_def() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Wrapper<T, const N: usize>(T, Option<[u8; N]>);

    let mut registry = Registry::new();
    let decl_string = <Wrapper<String, 32> as TypeInfo>::type_decl(&mut registry);
    let decl_u32 = <Wrapper<u32, 32> as TypeInfo>::type_decl(&mut registry);

    assert_eq!(decl_string.to_string(), "WrapperN32<String>");
    assert_eq!(decl_u32.to_string(), "WrapperN32<u32>");

    let ref_string = registry
        .get_registered::<Wrapper<String, 32>>()
        .unwrap()
        .type_ref;
    let ref_u32 = registry
        .get_registered::<Wrapper<u32, 32>>()
        .unwrap()
        .type_ref;
    assert_eq!(ref_string, ref_u32);

    let ty = registry.get_type(ref_string).unwrap();
    assert_eq!(ty.name, "WrapperN32");
    assert_eq!(ty.type_params.len(), 1);
    assert_eq!(ty.type_params[0].name, "T");
}

#[test]
fn nested_named_fields_register_concrete_dependencies_and_store_abstract_decl() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Node<T> {
        value: T,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Wrapper<T> {
        node: Node<T>,
    }

    let mut registry = Registry::new();
    let decl_string = <Wrapper<String> as TypeInfo>::type_decl(&mut registry);
    let decl_u32 = <Wrapper<u32> as TypeInfo>::type_decl(&mut registry);

    assert_eq!(decl_string.to_string(), "Wrapper<String>");
    assert_eq!(decl_u32.to_string(), "Wrapper<u32>");
    assert!(registry.get_registered::<Node<String>>().is_some());
    assert!(registry.get_registered::<Node<u32>>().is_some());

    let wrapper_ref = registry
        .get_registered::<Wrapper<String>>()
        .unwrap()
        .type_ref;
    let wrapper = registry.get_type(wrapper_ref).unwrap();
    let TypeDef::Struct(def) = &wrapper.def else {
        panic!("expected struct");
    };
    assert_eq!(def.fields[0].type_decl.to_string(), "Node<T>");
}

#[allow(dead_code)]
#[derive(TypeInfo)]
struct MutualA<T> {
    b: Option<alloc::boxed::Box<MutualB<T>>>,
    _marker: PhantomData<T>,
}

#[allow(dead_code)]
#[derive(TypeInfo)]
struct MutualB<T> {
    a: Option<alloc::boxed::Box<MutualA<T>>>,
    _marker: PhantomData<T>,
}

#[test]
fn dependency_hook_runs_after_current_type_is_cached() {
    let mut registry = Registry::new();
    let decl = <MutualA<String> as TypeInfo>::type_decl(&mut registry);

    assert_eq!(decl.to_string(), "MutualA<String>");
    assert!(registry.get_registered::<MutualA<String>>().is_some());
    assert!(registry.get_registered::<MutualB<String>>().is_some());
}

mod left {
    #[allow(dead_code)]
    #[derive(sails_type_registry::TypeInfo)]
    pub struct SharedNode<T> {
        pub value: T,
    }
}

mod right {
    #[allow(dead_code)]
    #[derive(sails_type_registry::TypeInfo)]
    pub struct SharedNode<T> {
        pub value: T,
    }

    #[allow(dead_code)]
    #[derive(sails_type_registry::TypeInfo)]
    pub struct SharedWrapper<T> {
        pub node: super::left::SharedNode<T>,
    }
}

#[test]
fn nested_named_field_uses_registry_unique_name() {
    let mut registry = Registry::new();
    let _ = <right::SharedNode<String> as TypeInfo>::type_decl(&mut registry);
    let _ = <right::SharedWrapper<String> as TypeInfo>::type_decl(&mut registry);

    let wrapper_ref = registry
        .get_registered::<right::SharedWrapper<String>>()
        .unwrap()
        .type_ref;
    let wrapper = registry.get_type(wrapper_ref).unwrap();
    let TypeDef::Struct(def) = &wrapper.def else {
        panic!("expected struct");
    };

    let TypeDecl::Named { name, .. } = &def.fields[0].type_decl else {
        panic!("expected named left::SharedNode<T>");
    };
    assert_ne!(name, "SharedNode");
    assert!(registry.types().any(|(_, ty)| ty.name == *name));
}

#[test]
fn nested_const_generic_named_field_uses_const_suffixed_lookup() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Inner<T, const N: usize>(T, [u8; N]);

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Outer<T> {
        inner: Inner<T, 32>,
    }

    let mut registry = Registry::new();
    let _ = <Outer<String> as TypeInfo>::type_decl(&mut registry);

    let outer_ref = registry.get_registered::<Outer<String>>().unwrap().type_ref;
    let outer = registry.get_type(outer_ref).unwrap();
    let TypeDef::Struct(def) = &outer.def else {
        panic!("expected struct");
    };

    let TypeDecl::Named { name, generics } = &def.fields[0].type_decl else {
        panic!("expected named InnerN32<T>");
    };
    assert_eq!(name, "InnerN32");
    assert_eq!(generics.len(), 1);
    assert_eq!(generics[0], TypeDecl::generic("T"));
}

mod domain {
    #[allow(dead_code)]
    #[derive(sails_type_registry::TypeInfo)]
    pub struct String {
        pub value: u32,
    }

    #[allow(dead_code)]
    #[derive(sails_type_registry::TypeInfo)]
    pub struct Vec<T> {
        pub value: T,
    }

    #[allow(dead_code)]
    #[derive(sails_type_registry::TypeInfo)]
    pub struct T {
        pub value: u32,
    }
}

#[test]
fn qualified_user_paths_with_builtin_names_stay_named() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Holder<T> {
        text: domain::String,
        values: domain::Vec<T>,
    }

    let mut registry = Registry::new();
    let _ = <Holder<u32> as TypeInfo>::type_decl(&mut registry);

    assert!(registry.get_registered::<domain::String>().is_some());
    assert!(registry.get_registered::<domain::Vec<u32>>().is_some());

    let holder_ref = registry.get_registered::<Holder<u32>>().unwrap().type_ref;
    let holder = registry.get_type(holder_ref).unwrap();
    let TypeDef::Struct(def) = &holder.def else {
        panic!("expected struct");
    };

    let TypeDecl::Named { name, generics } = &def.fields[0].type_decl else {
        panic!("expected named domain::String");
    };
    assert_eq!(name, "String");
    assert!(generics.is_empty());

    let TypeDecl::Named { name, generics } = &def.fields[1].type_decl else {
        panic!("expected named domain::Vec<T>");
    };
    assert_eq!(name, "Vec");
    assert_eq!(generics.len(), 1);
    assert_eq!(generics[0], TypeDecl::generic("T"));
}

#[test]
fn named_field_name_does_not_collide_with_type_param_name() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Holder<T> {
        named: domain::T,
        generic: T,
    }

    let mut registry = Registry::new();
    let _ = <domain::T as TypeInfo>::type_decl(&mut registry);
    let _ = <Holder<domain::T> as TypeInfo>::type_decl(&mut registry);

    let holder_ref = registry
        .get_registered::<Holder<domain::T>>()
        .unwrap()
        .type_ref;
    let holder = registry.get_type(holder_ref).unwrap();
    let TypeDef::Struct(def) = &holder.def else {
        panic!("expected struct");
    };

    let TypeDecl::Named { name, generics } = &def.fields[0].type_decl else {
        panic!("expected named domain::T");
    };
    assert_eq!(name, "T");
    assert!(generics.is_empty());

    let TypeDecl::Generic { name } = &def.fields[1].type_decl else {
        panic!("expected generic T");
    };
    assert_eq!(name, "T");
}

#[test]
fn declared_default_type_params_are_preserved_in_type_def() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Defaulted<T = String> {
        value: T,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<Defaulted>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert_eq!(ty.type_params.len(), 1);
    assert_eq!(ty.type_params[0].name, "T");
    assert_eq!(
        ty.type_params[0].ty,
        Some(TypeDecl::Primitive(sails_idl_ast::PrimitiveType::String))
    );
}

#[test]
fn named_default_type_params_are_preserved_in_type_def() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct DefaultValue {
        value: u32,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct Defaulted<T = DefaultValue> {
        value: T,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<Defaulted>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();

    assert!(registry.get_registered::<DefaultValue>().is_some());
    assert_eq!(ty.type_params.len(), 1);
    assert_eq!(ty.type_params[0].name, "T");
    assert_eq!(
        ty.type_params[0].ty,
        Some(TypeDecl::Named {
            name: "DefaultValue".into(),
            generics: alloc::vec![],
        })
    );
}

#[test]
fn cow_fields_lower_to_owned_target() {
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct CowHolder {
        value: Cow<'static, str>,
    }

    let mut registry = Registry::new();
    let type_ref = registry.register_type::<CowHolder>().unwrap();
    let ty = registry.get_type(type_ref).unwrap();
    let TypeDef::Struct(def) = &ty.def else {
        panic!("expected struct");
    };

    assert_eq!(
        def.fields[0].type_decl,
        TypeDecl::Primitive(sails_idl_ast::PrimitiveType::String)
    );
}
