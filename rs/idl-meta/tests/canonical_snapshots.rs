mod fixtures;

use fixtures::canvas_service;
use sails_idl_meta::{CanonicalizationContext, compute_interface_id};

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
