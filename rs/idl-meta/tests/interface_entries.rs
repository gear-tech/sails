mod fixtures;

use fixtures::canvas_service;
use sails_idl_meta::{
    CanonicalizationContext, EntryAssignmentError, EntryKind, build_entry_meta_with_async,
    canonical_entries, compute_interface_id,
};

#[test]
fn canonical_entry_ordering_is_stable() {
    let service = canvas_service();
    let ctx = CanonicalizationContext::default();
    let result = compute_interface_id(&service, &ctx).expect("canonicalization");

    let entries = canonical_entries(&result.envelope);
    let ordered_names: Vec<_> = entries.iter().map(|entry| entry.name.as_str()).collect();
    assert_eq!(
        ordered_names,
        &["ColorPoint", "PointStatus", "Points", "StatusChanged"]
    );
}

#[test]
fn entry_meta_assigns_ids_and_async_flags() {
    let service = canvas_service();
    let ctx = CanonicalizationContext::default();
    let result = compute_interface_id(&service, &ctx).expect("canonicalization");

    let meta = build_entry_meta_with_async(&result.envelope, |entry| {
        // Pretend only the `Points` query is async in this mock.
        entry.name == "Points" && entry.kind == EntryKind::Query
    })
    .expect("ids");

    assert_eq!(meta.len(), 4);
    assert_eq!(meta[0].entry_id, 0);
    assert_eq!(meta[1].entry_id, 1);
    assert_eq!(meta[2].entry_id, 2);
    assert_eq!(meta[3].entry_id, 3);
    assert!(meta[2].is_async);
    assert!(meta.iter().all(|entry| !entry.name.is_empty()));
}

#[test]
fn entry_meta_errors_on_overflow() {
    use sails_idl_meta::canonical::{CanonicalEnvelope, CanonicalService};

    let mut envelope = CanonicalEnvelope {
        service: CanonicalService {
            extends: Vec::new(),
            functions: Vec::new(),
            events: Vec::new(),
        },
        ..CanonicalEnvelope::default()
    };

    // Simulate overflow by crafting many fake entries.
    envelope
        .service
        .functions
        .extend((0..(u16::MAX as usize + 2)).map(|idx| {
            sails_idl_meta::canonical::CanonicalFunction {
                name: format!("F{idx}"),
                kind: sails_idl_meta::canonical::CanonicalFunctionKind::Command,
                params: Vec::new(),
                output: sails_idl_meta::canonical::CanonicalType::Tuple { items: Vec::new() },
                throws: None,
            }
        }));

    let err = build_entry_meta_with_async(&envelope, |_| false).expect_err("overflow should fail");
    assert_eq!(
        err,
        EntryAssignmentError::TooManyEntries(u16::MAX as usize + 2)
    );
}
