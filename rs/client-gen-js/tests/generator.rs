use sails_client_gen_js::JsClientGenerator;
use insta::assert_snapshot;

#[test]
fn test_demo_generation() {
    let idl = include_str!("../../../js/test/demo-v2/demo.idl");
    let generated = JsClientGenerator::from_idl(idl)
        .generate()
        .expect("generate ts client");

    assert_snapshot!("demo_mvp_generation", generated);
}

#[test]
fn test_type_naming_parity_generation() {
    let idl = include_str!("idls/type_naming_parity.idl");
    let generated = JsClientGenerator::from_idl(idl)
        .generate()
        .expect("generate ts client");

    assert_snapshot!("type_naming_parity_generation", generated);
}

