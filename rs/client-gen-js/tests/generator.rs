use insta::assert_snapshot;
use sails_client_gen_js::JsClientGenerator;

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

#[test]
fn test_partial_idl_generation() {
    let idl = include_str!("idls/partial.idl");
    let generated = JsClientGenerator::from_idl(idl)
        .generate()
        .expect("generate ts client");

    assert_snapshot!("partial_idl_generation", generated);
}

#[test]
fn test_aliases_generation() {
    let idl = include_str!("idls/aliases_works.idl");
    let generated = JsClientGenerator::from_idl(idl)
        .generate()
        .expect("generate ts client");

    assert_snapshot!("aliases_generation", generated);
}

#[test]
fn codec_selection() {
    let idl = include_str!("idls/codec.idl");
    let generated = JsClientGenerator::from_idl(idl)
        .generate()
        .expect("generate ts client");

    // Included: both_method, scale_only, both_query
    assert!(
        generated.contains("bothMethod"),
        "expected bothMethod to be present"
    );
    assert!(
        generated.contains("scaleOnly"),
        "expected scaleOnly to be present"
    );
    assert!(
        generated.contains("bothQuery"),
        "expected bothQuery to be present"
    );

    // Excluded: ethabi_only, ethabi_query
    assert!(
        !generated.contains("ethabiOnly"),
        "expected ethabiOnly to be filtered out, got:\n{generated}"
    );
    assert!(
        !generated.contains("ethabiQuery"),
        "expected ethabiQuery to be filtered out, got:\n{generated}"
    );
    assert!(
        generated.contains("subscribeToBothEventEvent"),
        "expected subscribeToBothEventEvent to be present"
    );
    assert!(
        generated.contains("subscribeToScaleOnlyEventEvent"),
        "expected subscribeToScaleOnlyEventEvent to be present"
    );
    assert!(
        !generated.contains("subscribeToEthabiOnlyEventEvent"),
        "expected subscribeToEthabiOnlyEventEvent to be filtered out, got:\n{generated}"
    );

    assert_snapshot!("codec_selection", generated);
}
