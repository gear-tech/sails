#[test]
fn gprogram_fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/gprogram_fails*.rs");
}
