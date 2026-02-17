use askama::Template;
use sails_idl_meta::*;

mod fixture;

#[test]
fn type_enum() {
    let ty = fixture::enum_variants_type();
    let idl = ty.render().unwrap();
    insta::assert_snapshot!(idl);
}

#[test]
fn type_alias() {
    let ty = Type {
        name: "MyAlias".to_string(),
        type_params: vec![],
        def: TypeDef::Alias(AliasDef {
            target: TypeDecl::Primitive(PrimitiveType::U32),
        }),
        docs: vec!["My alias docs".to_string()],
        annotations: vec![],
    };
    let idl = ty.render().unwrap();
    insta::assert_snapshot!(idl);
}

#[test]
fn idl_globals() {
    let doc = IdlDoc {
        globals: fixture::globals(),
        program: None,
        services: vec![],
    };
    let idl = doc.render().unwrap();
    insta::assert_snapshot!(idl);
}

#[test]
fn idl_program() {
    let doc = IdlDoc {
        globals: fixture::globals(),
        program: Some(fixture::program_unit()),
        services: vec![],
    };
    let idl = doc.render().unwrap();
    insta::assert_snapshot!(idl);
}

#[test]
fn idl_service() {
    let doc = IdlDoc {
        globals: fixture::globals(),
        program: None,
        services: vec![fixture::counter_service(), fixture::this_that_service()],
    };
    let idl = doc.render().unwrap();
    insta::assert_snapshot!(idl);
}
