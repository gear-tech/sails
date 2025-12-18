use sails_idl_meta::*;

mod fixture;

#[test]
fn type_enum() {
    let ty = fixture::enum_variants_type();
    let serialized = serde_json::to_string_pretty(&ty).unwrap();
    let value: Type = serde_json::from_str(&serialized).unwrap();

    assert_eq!(ty, value);
    insta::assert_snapshot!(serialized);
}

#[test]
fn service_unit() {
    let service = fixture::this_that_service();
    let serialized = serde_json::to_string_pretty(&service).unwrap();
    let value: ServiceUnit = serde_json::from_str(&serialized).unwrap();

    assert_eq!(service, value);
    insta::assert_snapshot!(serialized);
}

#[test]
fn program_unit() {
    let prg = fixture::program_unit();
    let serialized = serde_json::to_string_pretty(&prg).unwrap();
    let value: ProgramUnit = serde_json::from_str(&serialized).unwrap();

    assert_eq!(prg, value);
    insta::assert_snapshot!(serialized);
}

#[test]
fn idl_doc() {
    let doc = IdlDoc {
        globals: fixture::globals(),
        program: Some(fixture::program_unit()),
        services: vec![fixture::counter_service(), fixture::this_that_service()],
    };

    let serialized = serde_json::to_string_pretty(&doc).unwrap();
    let value: IdlDoc = serde_json::from_str(&serialized).unwrap();

    assert_eq!(doc, value);
    insta::assert_snapshot!(serialized);
}
