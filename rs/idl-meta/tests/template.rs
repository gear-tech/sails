use askama::Template;
use sails_idl_meta::*;

fn globals() -> Vec<(String, Option<String>)> {
    vec![
        ("sails".to_string(), Some("0.1.0".to_string())),
        ("include".to_string(), Some("ownable.idl".to_string())),
        (
            "include".to_string(),
            Some("git://github.com/some_repo/tippable.idl".to_string()),
        ),
    ]
}

fn counter_service() -> ServiceUnit {
    use PrimitiveType::*;
    use TypeDecl::*;

    ServiceUnit {
        name: "Counter".to_string(),
        extends: vec![],
        funcs: vec![
            ServiceFunc {
                name: "Add".to_string(),
                params: vec![FuncParam {
                    name: "value".to_string(),
                    type_decl: Primitive(U32),
                }],
                output: Primitive(U32),
                throws: None,
                is_query: false,
                docs: vec!["Add a value to the counter".to_string()],
                annotations: vec![],
            },
            ServiceFunc {
                name: "Sub".to_string(),
                params: vec![FuncParam {
                    name: "value".to_string(),
                    type_decl: Primitive(U32),
                }],
                output: Primitive(U32),
                throws: Some(Primitive(String)),
                is_query: false,
                docs: vec!["Substract a value from the counter".to_string()],
                annotations: vec![],
            },
            ServiceFunc {
                name: "Value".to_string(),
                params: vec![],
                output: Primitive(U32),
                throws: None,
                is_query: true,
                docs: vec!["Get the current value".to_string()],
                annotations: vec![("query".to_string(), None)],
            },
        ],
        events: vec![
            ServiceEvent {
                name: "Added".to_string(),
                def: StructDef {
                    fields: vec![StructField {
                        name: None,
                        type_decl: Primitive(U32),
                        docs: vec![],
                        annotations: vec![],
                    }],
                },
                docs: vec!["Emitted when a new value is added to the counter".to_string()],
                annotations: vec![],
            },
            ServiceEvent {
                name: "Subtracted".to_string(),
                def: StructDef {
                    fields: vec![StructField {
                        name: None,
                        type_decl: Primitive(U32),
                        docs: vec![],
                        annotations: vec![],
                    }],
                },
                docs: vec!["Emitted when a value is subtracted from the counter".to_string()],
                annotations: vec![],
            },
        ],
        types: vec![],
        docs: vec![],
        annotations: vec![],
    }
}

fn this_that_service() -> ServiceUnit {
    use PrimitiveType::*;
    use TypeDecl::*;
    use TypeDef::*;

    ServiceUnit {
        name: "ThisThat ".to_string(),
        extends: vec![],
        funcs: vec![ServiceFunc {
            name: "DoThis".to_string(),
            params: vec![
                FuncParam {
                    name: "p1".to_string(),
                    type_decl: Primitive(U32),
                },
                FuncParam {
                    name: "p2".to_string(),
                    type_decl: Primitive(String),
                },
                FuncParam {
                    name: "p3".to_string(),
                    type_decl: Tuple(vec![
                        Option(Box::new(Primitive(H160))),
                        UserDefined {
                            path: "NonZeroU8".to_string(),
                            generics: vec![],
                        },
                    ]),
                },
                FuncParam {
                    name: "p4".to_string(),
                    type_decl: UserDefined {
                        path: "TupleStruct".to_string(),
                        generics: vec![],
                    },
                },
            ],
            output: Tuple(vec![Primitive(String), Primitive(U32)]),
            throws: Some(Tuple(vec![Primitive(String)])),
            is_query: false,
            docs: vec!["Add a value to the counter".to_string()],
            annotations: vec![],
        }],
        events: vec![],
        types: vec![
            Type {
                name: "DoThatParam".to_string(),
                type_params: vec![],
                def: Struct(StructDef {
                    fields: vec![
                        StructField {
                            name: Some("p1".to_string()),
                            type_decl: Primitive(U32),
                            docs: vec!["Parametr p1: u32".to_string()],
                            annotations: vec![],
                        },
                        StructField {
                            name: Some("p2".to_string()),
                            type_decl: Primitive(ActorId),
                            docs: vec![],
                            annotations: vec![],
                        },
                        StructField {
                            name: Some("p3".to_string()),
                            type_decl: UserDefined {
                                path: "ManyVariants".to_string(),
                                generics: vec![],
                            },
                            docs: vec![],
                            annotations: vec![],
                        },
                    ],
                }),
                docs: vec![],
                annotations: vec![],
            },
            Type {
                name: "ManyVariants".to_string(),
                type_params: vec![],
                def: Enum(EnumDef {
                    variants: vec![
                        EnumVariant {
                            name: "One".to_string(),
                            def: StructDef { fields: vec![] },
                            docs: vec![],
                            annotations: vec![],
                        },
                        EnumVariant {
                            name: "Two".to_string(),
                            def: StructDef {
                                fields: vec![StructField {
                                    name: None,
                                    type_decl: Primitive(U32),
                                    docs: vec![],
                                    annotations: vec![],
                                }],
                            },
                            docs: vec![],
                            annotations: vec![],
                        },
                        EnumVariant {
                            name: "Three".to_string(),
                            def: StructDef {
                                fields: vec![StructField {
                                    name: None,
                                    type_decl: Option(Box::new(Primitive(U32))),
                                    docs: vec![],
                                    annotations: vec![],
                                }],
                            },
                            docs: vec![],
                            annotations: vec![],
                        },
                        EnumVariant {
                            name: "Four".to_string(),
                            def: StructDef {
                                fields: vec![
                                    StructField {
                                        name: Some("a".to_string()),
                                        type_decl: Primitive(U32),
                                        docs: vec![],
                                        annotations: vec![],
                                    },
                                    StructField {
                                        name: Some("b".to_string()),
                                        type_decl: Option(Box::new(Primitive(U16))),
                                        docs: vec![],
                                        annotations: vec![],
                                    },
                                ],
                            },
                            docs: vec![],
                            annotations: vec![],
                        },
                        EnumVariant {
                            name: "Five".to_string(),
                            def: StructDef {
                                fields: vec![
                                    StructField {
                                        name: None,
                                        type_decl: Primitive(String),
                                        docs: vec![],
                                        annotations: vec![],
                                    },
                                    StructField {
                                        name: None,
                                        type_decl: Primitive(H256),
                                        docs: vec![],
                                        annotations: vec![(
                                            "key".to_string(),
                                            Some("value".to_string()),
                                        )],
                                    },
                                ],
                            },
                            docs: vec![],
                            annotations: vec![],
                        },
                        EnumVariant {
                            name: "Six".to_string(),
                            def: StructDef {
                                fields: vec![StructField {
                                    name: None,
                                    type_decl: Tuple(vec![Primitive(U32)]),
                                    docs: vec![],
                                    annotations: vec![],
                                }],
                            },
                            docs: vec![],
                            annotations: vec![],
                        },
                    ],
                }),
                docs: vec![],
                annotations: vec![],
            },
        ],
        docs: vec![],
        annotations: vec![],
    }
}

#[test]
fn idl_globals() {
    let doc = IdlDoc {
        globals: globals(),
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
        globals: globals(),
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

#[test]
fn idl_service() {
    let doc = IdlDoc {
        globals: globals(),
        program: None,
        services: vec![counter_service(), this_that_service()],
    };
    let idl = doc.render().unwrap();
    insta::assert_snapshot!(idl);
}
