#[test]
fn ethapp_with_events_generate_idl() {
    let mut idl = String::new();
    sails_rs::generate_idl::<ethapp_with_events::MyProgram>(Some("MyProgram"), &mut idl).unwrap();
    insta::assert_snapshot!(idl);
}
