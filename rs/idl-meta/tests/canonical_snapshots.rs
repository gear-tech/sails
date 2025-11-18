mod fixtures;

use fixtures::{
    canvas_service, collision_base_service, collision_child_service, unused_type_service,
};
use sails_idl_meta::ast::{ServiceUnit, TypeDecl, TypeDef};
use sails_idl_meta::{
    AnyServiceMeta, CanonicalizationContext, ParentInterface, ServiceMeta, build_service_unit,
    compute_interface_id,
};
use scale_info::TypeInfo;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn canvas_service_snapshot() {
    let service = canvas_service();
    let ctx = CanonicalizationContext::default();
    let result = compute_interface_id(&service, &ctx).expect("canonicalization");
    let json = String::from_utf8(result.canonical_json.clone()).expect("utf8");
    insta::assert_snapshot!(
        "canvas_service_canonical",
        format!(
            "interface_id: 0x{interface_id:016x}\n{json}",
            interface_id = result.interface_id,
            json = json
        )
    );
}

#[derive(TypeInfo)]
pub enum CollectionsCommandsMeta {
    Upsert(UpsertParams, ()),
}

#[derive(TypeInfo)]
pub struct UpsertParams {
    pub parts: BTreeMap<u32, Part>,
}

#[derive(TypeInfo)]
pub struct Part {
    pub owners: BTreeSet<u64>,
}

#[derive(TypeInfo)]
pub enum CollectionsQueriesMeta {
    Snapshot(SnapshotParams, SnapshotResult),
}

#[derive(TypeInfo)]
pub struct SnapshotParams {}

#[derive(TypeInfo)]
pub struct SnapshotResult {
    pub balances: BTreeSet<u32>,
}

#[derive(TypeInfo)]
pub enum CollectionsEventsMeta {
    Synced(SyncedEvent),
}

#[derive(TypeInfo)]
pub struct SyncedEvent {
    pub ids: BTreeSet<u32>,
}

pub struct CollectionsServiceMeta;

impl ServiceMeta for CollectionsServiceMeta {
    type CommandsMeta = CollectionsCommandsMeta;
    type QueriesMeta = CollectionsQueriesMeta;
    type EventsMeta = CollectionsEventsMeta;
    const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
    const ASYNC: bool = false;
}

#[test]
fn collections_service_snapshot() {
    let meta = AnyServiceMeta::new::<CollectionsServiceMeta>();
    let unit = build_service_unit("Collections", &meta).expect("service ast");
    let ctx = CanonicalizationContext::default();
    let result = compute_interface_id(&unit, &ctx).expect("canonicalization");
    let json = String::from_utf8(result.canonical_json.clone()).expect("utf8");
    insta::assert_snapshot!(
        "collections_service_canonical",
        format!(
            "interface_id: 0x{interface_id:016x}\n{json}",
            interface_id = result.interface_id,
            json = json
        )
    );
}

#[test]
fn drops_unused_types_from_canonical_output() {
    let service = unused_type_service();
    let ctx = CanonicalizationContext::default();
    let result = compute_interface_id(&service, &ctx).expect("canonicalization");
    assert!(result.envelope.type_bindings.contains_key("self::Used"));
    assert!(!result.envelope.type_bindings.contains_key("self::Unused"));
}

#[test]
fn collision_service_snapshot() {
    let base = collision_base_service();
    let base_ctx = CanonicalizationContext::default();
    let base_result = compute_interface_id(&base, &base_ctx).expect("base canonicalization");
    let parent = ParentInterface::new(&base, base_result.interface_id);
    let parent_refs = [parent];
    let child = collision_child_service();
    let ctx = CanonicalizationContext::with_parents(&parent_refs);
    let result = compute_interface_id(&child, &ctx).expect("child canonicalization");
    let json = String::from_utf8(result.canonical_json.clone()).expect("utf8");
    insta::assert_snapshot!(
        "collision_service_canonical",
        format!(
            "interface_id: 0x{interface_id:016x}\n{json}",
            interface_id = result.interface_id,
            json = json
        )
    );
}

#[test]
fn renaming_service_name_does_not_change_interface_id() {
    let mut service = canvas_service();
    let ctx = CanonicalizationContext::default();
    let baseline = compute_interface_id(&service, &ctx).expect("baseline canonicalization");

    service.name = "CanvasRenamed".to_string();
    let renamed = compute_interface_id(&service, &ctx).expect("renamed canonicalization");
    assert_eq!(baseline.interface_id, renamed.interface_id);
}

#[test]
fn renaming_parent_name_does_not_change_child_interface_id() {
    let mut base = collision_base_service();
    let base_ctx = CanonicalizationContext::default();
    let base_result = compute_interface_id(&base, &base_ctx).expect("base canonicalization");

    let mut child = collision_child_service();
    let parent = ParentInterface::new(&base, base_result.interface_id);
    let parent_refs = [parent];
    let child_ctx = CanonicalizationContext::with_parents(&parent_refs);
    let baseline_child = compute_interface_id(&child, &child_ctx).expect("child canonicalization");

    let new_parent_name = "CollisionBaseRenamed".to_string();
    rename_service(&mut base, &new_parent_name);
    rename_parent_references(&mut child, "CollisionBase", &new_parent_name);

    let renamed_base =
        compute_interface_id(&base, &base_ctx).expect("renamed base canonicalization");
    assert_eq!(base_result.interface_id, renamed_base.interface_id);

    let renamed_parent = ParentInterface::new(&base, renamed_base.interface_id);
    let renamed_refs = [renamed_parent];
    let renamed_ctx = CanonicalizationContext::with_parents(&renamed_refs);
    let renamed_child =
        compute_interface_id(&child, &renamed_ctx).expect("renamed child canonicalization");
    assert_eq!(baseline_child.interface_id, renamed_child.interface_id);
}

#[test]
fn renaming_type_name_does_not_change_interface_id() {
    let mut service = canvas_service();
    let ctx = CanonicalizationContext::default();
    let baseline = compute_interface_id(&service, &ctx).expect("baseline canonicalization");

    rename_type_definition(&mut service, "Point", "PointRenamed");
    let renamed = compute_interface_id(&service, &ctx).expect("renamed canonicalization");

    assert_eq!(baseline.interface_id, renamed.interface_id);
}

#[test]
fn renaming_parent_type_name_does_not_change_child_interface_id() {
    let mut base = collision_base_service();
    let mut child = collision_child_service();
    let base_ctx = CanonicalizationContext::default();
    let base_result = compute_interface_id(&base, &base_ctx).expect("base canonicalization");
    let baseline_child_id = {
        let parent = ParentInterface::new(&base, base_result.interface_id);
        let parents = [parent];
        let ctx = CanonicalizationContext::with_parents(&parents);
        compute_interface_id(&child, &ctx)
            .expect("child canonicalization")
            .interface_id
    };

    rename_type_definition(&mut base, "Shared", "SharedRenamed");
    let renamed_base =
        compute_interface_id(&base, &base_ctx).expect("renamed base canonicalization");
    assert_eq!(base_result.interface_id, renamed_base.interface_id);

    rename_type_references(
        &mut child,
        "CollisionBase::Shared",
        "CollisionBase::SharedRenamed",
    );
    let renamed_parent = ParentInterface::new(&base, renamed_base.interface_id);
    let renamed_parents = [renamed_parent];
    let renamed_ctx = CanonicalizationContext::with_parents(&renamed_parents);
    let renamed_child =
        compute_interface_id(&child, &renamed_ctx).expect("renamed child canonicalization");
    assert_eq!(baseline_child_id, renamed_child.interface_id);
}

fn rename_service(service: &mut ServiceUnit, new_name: &str) {
    service.name = new_name.to_string();
}

fn rename_parent_references(service: &mut ServiceUnit, old: &str, new: &str) {
    for extends in &mut service.extends {
        if extends == old {
            *extends = new.to_string();
        }
    }
    rename_type_references(service, old, new);
}

fn rename_type_definition(service: &mut ServiceUnit, old: &str, new: &str) {
    rename_type_references(service, old, new);
    for ty in &mut service.types {
        if ty.name == old {
            ty.name = new.to_string();
        }
    }
}

fn rename_type_references(service: &mut ServiceUnit, old: &str, new: &str) {
    for func in &mut service.funcs {
        for param in &mut func.params {
            rename_type_decl(&mut param.type_decl, old, new);
        }
        rename_type_decl(&mut func.output, old, new);
        if let Some(throws) = &mut func.throws {
            rename_type_decl(throws, old, new);
        }
    }
    for event in &mut service.events {
        for field in &mut event.def.fields {
            rename_type_decl(&mut field.type_decl, old, new);
        }
    }
    for ty in &mut service.types {
        match &mut ty.def {
            TypeDef::Struct(def) => {
                for field in &mut def.fields {
                    rename_type_decl(&mut field.type_decl, old, new);
                }
            }
            TypeDef::Enum(enum_def) => {
                for variant in &mut enum_def.variants {
                    for field in &mut variant.def.fields {
                        rename_type_decl(&mut field.type_decl, old, new);
                    }
                }
            }
        }
    }
}

fn rename_type_decl(ty: &mut TypeDecl, old: &str, new: &str) {
    match ty {
        TypeDecl::Slice(inner) => rename_type_decl(inner, old, new),
        TypeDecl::Array(inner, ..) => rename_type_decl(inner, old, new),
        TypeDecl::Tuple(items) => {
            for item in items {
                rename_type_decl(item, old, new);
            }
        }
        TypeDecl::Named(name, generics) => {
            if let Some(rest) = name
                .strip_prefix(old)
                .and_then(|suffix| suffix.strip_prefix("::"))
            {
                *name = format!("{new}::{rest}");
            } else if name == old {
                *name = new.to_string();
            }
            for arg in generics {
                rename_type_decl(arg, old, new);
            }
        }
        TypeDecl::Primitive(_) => {}
    }
}
