#[test]
fn validation_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/validation_gprogram_fails_for_solidity_reserved_name.rs");
    t.compile_fail("tests/ui/validation_gprogram_fails_for_service_constructor_reserved_name.rs");
    t.pass("tests/ui/validation_passes_for_non_exported_name.rs");
}
