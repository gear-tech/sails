use askama::Template;
use sails_idl_meta::*;

#[test]
fn idl_globals() {
    let doc = IdlDoc {
        globals: vec![
            ("sails".to_string(), Some("0.1.0".to_string())),
            ("include".to_string(), Some("ownable.idl".to_string())),
            (
                "include".to_string(),
                Some("git://github.com/some_repo/tippable.idl".to_string()),
            ),
        ],
        program: None,
        services: vec![],
    };
    let idl = doc.render().unwrap();
    insta::assert_snapshot!(idl);
}

#[test]
fn idl_program() {
    use PrimitiveType::*;
    use TypeDecl::*;
    use TypeDef::*;

    let doc = IdlDoc {
        globals: vec![
            ("sails".to_string(), Some("0.1.0".to_string())),
            ("include".to_string(), Some("ownable.idl".to_string())),
            (
                "include".to_string(),
                Some("git://github.com/some_repo/tippable.idl".to_string()),
            ),
        ],
        program: Some(ProgramUnit {
            name: "Demo".to_string(),
            ctors: vec![
                CtorFunc {
                    name: "Create".to_string(),
                    params: vec![FuncParam { name: "counter".to_string(), type_decl: Option(Box::new(Primitive(U32))) },
                        FuncParam { name: "dog_position".to_string(), type_decl: Option(Box::new(Tuple(vec![Primitive(I32), Primitive(I32)]))) }],
                    docs: vec!["Program constructor (called once at the very beginning of the program lifetime)".to_string()],
                    annotations: vec![],
                },
                CtorFunc {
                    name: "Default".to_string(),
                    params: vec![],
                    docs: vec!["Another program constructor".to_string(), "(called once at the very beginning of the program lifetime)".to_string()],
                    annotations: vec![],
                },
            ],
            services: vec![ProgramServiceItem { name: "Ping".to_string(), route: None, docs: vec![], annotations: vec![] },
                ProgramServiceItem { name: "Counter".to_string(), route: None, docs: vec![], annotations: vec![] },
                ProgramServiceItem { name: "Counter".to_string(), route: Some("Counter2".to_string()), docs: vec!["Another Counter service".to_owned()], annotations: vec![] }],
            types: vec![
                Type { name: "DoThatParam".to_string(), type_params: vec![], def: Struct(StructDef {fields:  vec![
                    StructField { name: Some("p1".to_string()), type_decl: Primitive(U32), docs: vec![], annotations: vec![] },
                    StructField { name: Some("p2".to_string()), type_decl: Primitive(ActorId), docs: vec![], annotations: vec![] },
                    StructField { name: Some("p3".to_string()), type_decl: UserDefined { path: "ManyVariants".to_string(), generics: vec![] }, docs: vec![], annotations: vec![] },
                ]}) , docs: vec![], annotations: vec![] },
                Type { name: "TupleStruct".to_string(), type_params: vec![], def: Struct(StructDef {fields:  vec![
                    StructField { name: None, type_decl: Primitive(U32), docs: vec![], annotations: vec![] },
                ]}) , docs: vec![], annotations: vec![] },
                Type { name: "UnitStruct".to_string(), type_params: vec![], def: Struct(StructDef {fields:  vec![]}) , docs: vec![], annotations: vec![] },
            ],
            docs: vec!["Demo Program".to_string()],
            annotations: vec![],
        }),
        services: vec![],
    };
    let idl = doc.render().unwrap();
    insta::assert_snapshot!(idl);
}
