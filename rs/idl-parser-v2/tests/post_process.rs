use sails_idl_parser_v2::ast::TypeDef;
use sails_idl_parser_v2::parse_idl;

#[test]
fn validate_named_types_fails_on_unknown_type() {
    let src = include_str!("idls/post_process_unknown_type.idl");
    let err = parse_idl(src).expect_err("Should have failed due to unknown type");
    assert!(err.to_string().contains("Unknown type 'UnknownType'"));
}

#[test]
fn validate_named_types_fails_on_unknown_generic_param() {
    let src = include_str!("idls/post_process_unknown_generic.idl");
    let err = parse_idl(src).expect_err("Should have failed due to unknown generic parameter");
    assert!(err.to_string().contains("Unknown type 'UnknownGeneric'"));
}

#[test]
fn validate_named_types_fails_on_service_type_in_program() {
    let src = include_str!("idls/post_process_service_type_in_program.idl");
    let err =
        parse_idl(src).expect_err("Should have failed due to service type used in program scope");
    assert!(err.to_string().contains("Unknown type 'ServiceType'"));
}

#[test]

fn validate_named_types_allows_program_type_in_service() {
    let src = include_str!("idls/post_process_program_type_in_service.idl");

    parse_idl(src).expect("Should successfully parse IDL with program type used in service");
}

#[test]

fn validate_scoping_fails_on_sibling_service_type_usage() {
    let src = include_str!("idls/post_process_advanced_scoping.idl");

    let result = parse_idl(src);
    println!("{result:?}");
    let err = result.expect_err("Should have failed due to sibling service type usage");
    assert!(err.to_string().contains("Unknown type 'TypeX'"));
}

#[test]
fn validate_mixed_fields_fails() {
    let src = include_str!("idls/post_process_mixed_fields.idl");
    let err = parse_idl(src).expect_err("Should have failed due to mixed fields");
    assert!(
        err.to_string().contains(
            "Mixing named and unnamed fields in a struct or enum variant is not allowed."
        )
    );
}

#[test]
fn parse_doc_annotation_works() {
    const SRC: &str = include_str!("../tests/idls/doc_annotation.idl");
    let doc = parse_idl(SRC).expect("parse idl");

    let program = doc.program.as_ref().unwrap();
    assert_eq!(
        program.docs,
        vec![
            "1. This is a program doc comment.",
            "2. This is also a program doc comment."
        ]
    );

    let ctor = &program.ctors[0];
    assert_eq!(
        ctor.docs,
        vec![
            "1. This is a ctor doc comment.",
            "2. This is also a ctor doc comment."
        ]
    );

    let ty = &program.types[0];
    assert_eq!(
        ty.docs,
        vec![
            "1. This is a struct doc comment.",
            "2. This is also a struct doc comment."
        ]
    );

    let field = match &ty.def {
        TypeDef::Struct(s) => &s.fields[0],
        _ => panic!("Expected struct"),
    };
    assert_eq!(
        field.docs,
        vec![
            "1. This is a field doc comment.",
            "2. This is also a field doc comment."
        ]
    );
}
