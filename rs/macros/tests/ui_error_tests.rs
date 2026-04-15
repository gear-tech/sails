#![cfg(not(feature = "ethexe"))]

#[test]
fn gservice_fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/gservice_fails*.rs");
}

#[test]
fn gprogram_fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/gprogram_fails*.rs");
}

#[test]
fn sails_type_fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/sails_type_fails*.rs");
}
