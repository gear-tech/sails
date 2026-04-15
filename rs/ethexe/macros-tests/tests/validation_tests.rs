#[test]
fn validation_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/validation_gservice_fails_export_duplicate_ethabi.rs");
    t.compile_fail("tests/ui/validation_gservice_fails_export_payable_without_ethabi.rs");
    t.compile_fail("tests/ui/validation_gprogram_fails_for_solidity_reserved_name.rs");
    t.compile_fail("tests/ui/validation_gprogram_fails_for_service_constructor_reserved_name.rs");
    t.pass("tests/ui/validation_passes_for_non_exported_name.rs");
}

#[test]
fn ethabi_only_non_scale_type_passes() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/validation_passes_ethabi_only_non_scale_type.rs");
}
