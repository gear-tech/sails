// Verifies the `EnvWithCtor` capability gate under the `ethexe` feature:
// `GstdEnv` deploys via the L1, so it cannot create programs from within a
// program and any constructor use must fail to compile (not panic at runtime).
#[test]
fn env_with_ctor_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/ethexe_gstd_env_without_ctor.rs");
}
