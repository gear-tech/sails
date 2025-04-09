#[test]
fn ethapp_with_events_generate_idl() {
    let mut idl = Vec::new();
    sails_rs::generate_idl::<ethapp_with_events::MyProgram>(&mut idl).unwrap();
    let idl = String::from_utf8(idl).unwrap();
    insta::assert_snapshot!(idl);
}
