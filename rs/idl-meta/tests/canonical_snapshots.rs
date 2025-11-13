use sails_idl_meta::ast::*;
use sails_idl_meta::{CanonicalizationContext, compute_interface_id};

fn sample_service() -> ServiceUnit {
    ServiceUnit {
        name: "Canvas".to_string(),
        extends: Vec::new(),
        funcs: vec![
            ServiceFunc {
                name: "ColorPoint".to_string(),
                params: vec![
                    FuncParam {
                        name: "point".to_string(),
                        type_decl: TypeDecl::UserDefined {
                            name: "Point".to_string(),
                            generics: Vec::new(),
                        },
                    },
                    FuncParam {
                        name: "color".to_string(),
                        type_decl: TypeDecl::UserDefined {
                            name: "Color".to_string(),
                            generics: Vec::new(),
                        },
                    },
                ],
                output: TypeDecl::Primitive(PrimitiveType::Bool),
                throws: Some(TypeDecl::UserDefined {
                    name: "ColorError".to_string(),
                    generics: Vec::new(),
                }),
                is_query: false,
                docs: Vec::new(),
                annotations: Vec::new(),
            },
            ServiceFunc {
                name: "Points".to_string(),
                params: vec![
                    FuncParam {
                        name: "offset".to_string(),
                        type_decl: TypeDecl::Primitive(PrimitiveType::U32),
                    },
                    FuncParam {
                        name: "len".to_string(),
                        type_decl: TypeDecl::Primitive(PrimitiveType::U32),
                    },
                ],
                output: TypeDecl::UserDefined {
                    name: "PointStatus".to_string(),
                    generics: Vec::new(),
                },
                throws: Some(TypeDecl::Primitive(PrimitiveType::String)),
                is_query: true,
                docs: Vec::new(),
                annotations: Vec::new(),
            },
            ServiceFunc {
                name: "PointStatus".to_string(),
                params: vec![FuncParam {
                    name: "point".to_string(),
                    type_decl: TypeDecl::UserDefined {
                        name: "Point".to_string(),
                        generics: Vec::new(),
                    },
                }],
                output: TypeDecl::Option(Box::new(TypeDecl::UserDefined {
                    name: "PointStatus".to_string(),
                    generics: Vec::new(),
                })),
                throws: None,
                is_query: true,
                docs: Vec::new(),
                annotations: Vec::new(),
            },
        ],
        events: vec![EnumVariant {
            name: "StatusChanged".to_string(),
            def: StructDef {
                fields: vec![
                    StructField {
                        name: Some("point".to_string()),
                        type_decl: TypeDecl::UserDefined {
                            name: "Point".to_string(),
                            generics: Vec::new(),
                        },
                        docs: Vec::new(),
                        annotations: Vec::new(),
                    },
                    StructField {
                        name: Some("color".to_string()),
                        type_decl: TypeDecl::UserDefined {
                            name: "Color".to_string(),
                            generics: Vec::new(),
                        },
                        docs: Vec::new(),
                        annotations: Vec::new(),
                    },
                ],
            },
            docs: Vec::new(),
            annotations: Vec::new(),
        }],
        types: vec![
            Type {
                name: "Point".to_string(),
                type_params: Vec::new(),
                def: TypeDef::Struct(StructDef {
                    fields: vec![
                        StructField {
                            name: Some("x".to_string()),
                            type_decl: TypeDecl::Primitive(PrimitiveType::U32),
                            docs: Vec::new(),
                            annotations: Vec::new(),
                        },
                        StructField {
                            name: Some("y".to_string()),
                            type_decl: TypeDecl::Primitive(PrimitiveType::U32),
                            docs: Vec::new(),
                            annotations: Vec::new(),
                        },
                    ],
                }),
                docs: Vec::new(),
                annotations: Vec::new(),
            },
            Type {
                name: "Color".to_string(),
                type_params: Vec::new(),
                def: TypeDef::Struct(StructDef {
                    fields: vec![StructField {
                        name: Some("rgba".to_string()),
                        type_decl: TypeDecl::Array {
                            item: Box::new(TypeDecl::Primitive(PrimitiveType::U8)),
                            len: 4,
                        },
                        docs: Vec::new(),
                        annotations: Vec::new(),
                    }],
                }),
                docs: Vec::new(),
                annotations: Vec::new(),
            },
            Type {
                name: "ColorError".to_string(),
                type_params: Vec::new(),
                def: TypeDef::Enum(EnumDef {
                    variants: vec![
                        EnumVariant {
                            name: "InvalidSource".to_string(),
                            def: StructDef { fields: Vec::new() },
                            docs: Vec::new(),
                            annotations: Vec::new(),
                        },
                        EnumVariant {
                            name: "DeadPoint".to_string(),
                            def: StructDef { fields: Vec::new() },
                            docs: Vec::new(),
                            annotations: Vec::new(),
                        },
                    ],
                }),
                docs: Vec::new(),
                annotations: Vec::new(),
            },
            Type {
                name: "PointStatus".to_string(),
                type_params: Vec::new(),
                def: TypeDef::Enum(EnumDef {
                    variants: vec![
                        EnumVariant {
                            name: "Colored".to_string(),
                            def: StructDef {
                                fields: vec![
                                    StructField {
                                        name: Some("author".to_string()),
                                        type_decl: TypeDecl::Primitive(PrimitiveType::ActorId),
                                        docs: Vec::new(),
                                        annotations: Vec::new(),
                                    },
                                    StructField {
                                        name: Some("color".to_string()),
                                        type_decl: TypeDecl::UserDefined {
                                            name: "Color".to_string(),
                                            generics: Vec::new(),
                                        },
                                        docs: Vec::new(),
                                        annotations: Vec::new(),
                                    },
                                ],
                            },
                            docs: Vec::new(),
                            annotations: Vec::new(),
                        },
                        EnumVariant {
                            name: "Dead".to_string(),
                            def: StructDef { fields: Vec::new() },
                            docs: Vec::new(),
                            annotations: Vec::new(),
                        },
                    ],
                }),
                docs: Vec::new(),
                annotations: Vec::new(),
            },
        ],
        docs: Vec::new(),
        annotations: Vec::new(),
    }
}

#[test]
fn canvas_service_snapshot() {
    let service = sample_service();
    let ctx = CanonicalizationContext::default();
    let result = compute_interface_id(&service, &ctx).expect("canonicalization");
    let json = String::from_utf8(result.canonical_json.clone()).expect("utf8");
    insta::assert_snapshot!(
        "canvas_service_canonical",
        format!(
            "interface_id: 0x{interface_id:016x}\n{json}",
            interface_id = result.interface_id,
            json = json
        )
    );
}
