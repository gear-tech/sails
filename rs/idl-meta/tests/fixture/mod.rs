#![allow(unused)]

use sails_idl_meta::{PrimitiveType::*, TypeDecl::*, TypeDef::*, *};
use std::string::String;

pub fn globals() -> Vec<(String, Option<String>)> {
    vec![
        ("sails".to_string(), Some("0.1.0".to_string())),
        ("include".to_string(), Some("ownable.idl".to_string())),
        (
            "include".to_string(),
            Some("git://github.com/some_repo/tippable.idl".to_string()),
        ),
    ]
}

pub fn enum_variants_type() -> Type {
    Type {
        name: "SomeType".to_string(),
        type_params: vec![
            TypeParameter {
                name: "T1".to_string(),
                ty: None,
            },
            TypeParameter {
                name: "T2".to_string(),
                ty: None,
            },
        ],
        def: TypeDef::Enum(EnumDef {
            variants: vec![
                EnumVariant {
                    name: "Unit".to_string(),
                    def: StructDef { fields: vec![] },
                    docs: vec!["Unit-like Variant".to_string()],
                    annotations: vec![],
                },
                EnumVariant {
                    name: "Tuple".to_string(),
                    def: StructDef {
                        fields: vec![StructField {
                            name: None,
                            type_decl: Primitive(U32),
                            docs: vec![],
                            annotations: vec![],
                        }],
                    },
                    docs: vec!["Tuple-like Variant".to_string()],
                    annotations: vec![],
                },
                EnumVariant {
                    name: "TupleWithDocs".to_string(),
                    def: StructDef {
                        fields: vec![
                            StructField {
                                name: None,
                                type_decl: TypeDecl::option(Primitive(U32)),
                                docs: vec!["Some docs".to_string()],
                                annotations: vec![],
                            },
                            StructField {
                                name: None,
                                type_decl: TypeDecl::tuple(vec![Primitive(U32), Primitive(U32)]),
                                docs: vec!["Some docs".to_string()],
                                annotations: vec![],
                            },
                        ],
                    },
                    docs: vec!["Tuple-like Variant with field docs".to_string()],
                    annotations: vec![],
                },
                EnumVariant {
                    name: "Struct".to_string(),
                    def: StructDef {
                        fields: vec![
                            StructField {
                                name: Some("p1".to_string()),
                                type_decl: TypeDecl::option(Primitive(U32)),
                                docs: vec![],
                                annotations: vec![],
                            },
                            StructField {
                                name: Some("p2".to_string()),
                                type_decl: TypeDecl::tuple(vec![Primitive(U32), Primitive(U32)]),
                                docs: vec![],
                                annotations: vec![],
                            },
                        ],
                    },
                    docs: vec!["Struct-like Variant".to_string()],
                    annotations: vec![],
                },
                EnumVariant {
                    name: "GenericStruct".to_string(),
                    def: StructDef {
                        fields: vec![
                            StructField {
                                name: Some("p1".to_string()),
                                type_decl: TypeDecl::option(TypeDecl::named("T1".to_string())),
                                docs: vec![],
                                annotations: vec![],
                            },
                            StructField {
                                name: Some("p2".to_string()),
                                type_decl: TypeDecl::tuple(vec![
                                    TypeDecl::named("T2".to_string()),
                                    TypeDecl::named("T2".to_string()),
                                ]),
                                docs: vec![],
                                annotations: vec![],
                            },
                        ],
                    },
                    docs: vec!["Generic Struct-like Variant".to_string()],
                    annotations: vec![],
                },
            ],
        }),
        docs: vec!["SomeType Enum".to_string()],
        annotations: vec![(
            "rusttype".to_string(),
            Some("sails-idl-meta::SomeType".to_string()),
        )],
    }
}

pub fn counter_service() -> ServiceUnit {
    ServiceUnit {
        name: "Counter".parse().unwrap(),
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
                kind: FunctionKind::Command,
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
                throws: None,
                kind: FunctionKind::Command,
                docs: vec!["Substract a value from the counter".to_string()],
                annotations: vec![],
            },
            ServiceFunc {
                name: "Value".to_string(),
                params: vec![],
                output: Primitive(U32),
                throws: None,
                kind: FunctionKind::Query,
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

pub fn service_func() -> ServiceFunc {
    ServiceFunc {
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
                type_decl: TypeDecl::tuple(vec![
                    TypeDecl::option(Primitive(H160)),
                    TypeDecl::Named {
                        name: "NonZero".to_string(),
                        generics: vec![Primitive(U8)],
                    },
                ]),
            },
            FuncParam {
                name: "p4".to_string(),
                type_decl: TypeDecl::named("TupleStruct".to_string()),
            },
        ],
        output: TypeDecl::tuple(vec![Primitive(String), Primitive(U32)]),
        throws: Some(TypeDecl::tuple(vec![Primitive(String)])),
        kind: FunctionKind::Command,
        docs: vec!["Some func".to_string(), "With multiline doc".to_string()],
        annotations: vec![],
    }
}

pub fn this_that_service() -> ServiceUnit {
    ServiceUnit {
        name: "ThisThat".parse().unwrap(),
        extends: vec![],
        funcs: vec![service_func()],
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
                            type_decl: TypeDecl::named("ManyVariants".to_string()),
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
                                    type_decl: TypeDecl::option(Primitive(U32)),
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
                                        type_decl: TypeDecl::option(Primitive(U16)),
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
                                    type_decl: TypeDecl::tuple(vec![Primitive(U32)]),
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

pub fn ctor_func() -> CtorFunc {
    CtorFunc {
        name: "Create".to_string(),
        params: vec![
            FuncParam {
                name: "counter".to_string(),
                type_decl: TypeDecl::option(Primitive(U32)),
            },
            FuncParam {
                name: "dog_position".to_string(),
                type_decl: TypeDecl::option(TypeDecl::tuple(vec![Primitive(I32), Primitive(I32)])),
            },
        ],
        docs: vec![
            "Program constructor (called once at the very beginning of the program lifetime)"
                .to_string(),
        ],
        annotations: vec![],
    }
}

pub fn program_unit() -> ProgramUnit {
    ProgramUnit {
        name: "Demo".to_string(),
        ctors: vec![
            ctor_func(),
            CtorFunc {
                name: "Default".to_string(),
                params: vec![],
                docs: vec![
                    "Another program constructor".to_string(),
                    "(called once at the very beginning of the program lifetime)".to_string(),
                ],
                annotations: vec![],
            },
        ],
        services: vec![
            ServiceExpo {
                name: "Ping".parse().unwrap(),
                route: None,
                route_idx: 1,
                docs: vec![],
                annotations: vec![],
            },
            ServiceExpo {
                name: "Counter".parse().unwrap(),
                route: None,
                route_idx: 2,
                docs: vec![],
                annotations: vec![],
            },
            ServiceExpo {
                name: "Counter".parse().unwrap(),
                route: Some("Counter2".to_string()),
                route_idx: 3,
                docs: vec!["Another Counter service".to_owned()],
                annotations: vec![],
            },
        ],
        types: vec![
            Type {
                name: "DoThatParam".to_string(),
                type_params: vec![],
                def: Struct(StructDef {
                    fields: vec![
                        StructField {
                            name: Some("p1".to_string()),
                            type_decl: Primitive(U32),
                            docs: vec![],
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
                            type_decl: TypeDecl::named("ManyVariants".to_string()),
                            docs: vec![],
                            annotations: vec![],
                        },
                    ],
                }),
                docs: vec![],
                annotations: vec![],
            },
            type_tuple_struct(),
            type_unit_struct(),
        ],
        docs: vec!["Demo Program".to_string()],
        annotations: vec![],
    }
}

pub fn type_unit_struct() -> Type {
    Type {
        name: "UnitStruct".to_string(),
        type_params: vec![],
        def: Struct(StructDef { fields: vec![] }),
        docs: vec![],
        annotations: vec![],
    }
}

pub fn type_tuple_struct() -> Type {
    Type {
        name: "TupleStruct".to_string(),
        type_params: vec![],
        def: Struct(StructDef {
            fields: vec![StructField {
                name: None,
                type_decl: Primitive(U32),
                docs: vec![],
                annotations: vec![],
            }],
        }),
        docs: vec![],
        annotations: vec![],
    }
}
