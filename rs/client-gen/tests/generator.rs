use sails_client_gen::ClientGenerator;

#[test]
fn test_basic_works() {
    let idl = include_str!("idls/basic_works.idl");

    insta::assert_snapshot!(gen_client(idl, "Basic"));
}

#[test]
fn test_complex_type_generation_works() {
    const IDL: &str = include_str!("idls/complex_type_generation_works.idl");

    insta::assert_snapshot!(gen_client(IDL, "ComplexTypesProgram"));
}

#[test]
fn test_scope_resolution() {
    const IDL: &str = include_str!("idls/scope_test.idl");

    insta::assert_snapshot!(gen_client(IDL, "MyProgram"));
}

#[test]
fn test_multiple_services() {
    let idl = include_str!("idls/multiple_services.idl");

    insta::assert_snapshot!(gen_client(idl, "Multiple"));
}

#[test]
fn test_rmrk_works() {
    const IDL: &str = include_str!("idls/rmrk_works.idl");

    insta::assert_snapshot!(gen_client(IDL, "RmrkCatalog"));
}

#[test]
fn test_events_works() {
    let idl = include_str!("idls/events_works.idl");

    insta::assert_snapshot!(gen_client(idl, "ServiceWithEvents"));
}

#[test]
fn full_with_sails_path() {
    const IDL: &str = include_str!("idls/full_coverage.idl");

    let code = ClientGenerator::from_idl(IDL)
        .with_sails_crate("my_crate::sails")
        .generate("FullCoverageProgram") // Use new program name
        .expect("generate client");
    insta::assert_snapshot!(code);
}

#[test]
fn test_external_types() {
    const IDL: &str = include_str!("idls/external_types.idl");

    let code = ClientGenerator::from_idl(IDL)
        .with_sails_crate("my_crate::sails")
        .with_external_type("MyParam", "my_crate::MyParam")
        .with_no_derive_traits()
        .generate("Service")
        .expect("generate client");
    insta::assert_snapshot!(code);
}

fn gen_client(program: &str, service_name: &str) -> String {
    ClientGenerator::from_idl(program)
        .with_mocks("with_mocks")
        .generate(service_name)
        .expect("generate client")
}
