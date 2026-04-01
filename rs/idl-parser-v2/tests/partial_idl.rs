use sails_idl_parser_v2::parse_idl;

#[test]
fn validate_partial_service_success() {
    let src = include_str!("idls/partial_success.idl");
    let doc = parse_idl(src).expect("Should parse successfully with @partial and explicit ID");

    let service = &doc.services[0];
    let interface_id = service
        .name
        .interface_id
        .expect("interface_id must be present");
    assert_eq!(interface_id.to_string(), "0x1234567890abcdef");
}

#[test]
fn validate_partial_service_fails_without_entry_id() {
    let src = include_str!("idls/partial_fail_no_entry_id.idl");
    let err = parse_idl(src)
        .expect_err("Should fail due to missing @entry-id on function in @partial service");
    assert!(
        err.to_string()
            .contains("is missing `@entry-id` annotation (required for @partial services)"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_partial_service_fails_without_id() {
    let src = include_str!("idls/partial_fail.idl");
    let err =
        parse_idl(src).expect_err("Should fail due to missing explicit ID for @partial service");
    assert!(
        err.to_string()
            .contains("is marked as `@partial` but does not have an explicit `interface_id`")
    );
}
