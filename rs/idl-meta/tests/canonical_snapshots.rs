mod fixtures;

use fixtures::canvas_service;
use sails_idl_meta::{AnyServiceMeta, CanonicalizationContext, ServiceMeta, build_service_unit, compute_interface_id};
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
