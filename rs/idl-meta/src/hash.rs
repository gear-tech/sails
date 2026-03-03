use super::*;
use alloc::collections::btree_map::BTreeMap;
use keccak_const::Keccak256;

type Error = String;

impl ServiceUnit {
    /// Compute a deterministic interface identifier for this service.
    ///
    /// The hash incorporates:
    /// - all functions (kind, name, params, output, optional throws),
    /// - all events (their payload shape),
    /// - all base services by their already-computed interface IDs.
    ///
    /// Types referenced by functions or events are expanded via the AST
    /// definitions in `self.types`, including generic instantiation.
    pub fn interface_id(&self) -> Result<InterfaceId, Error> {
        let type_map: BTreeMap<_, _> = self.types.iter().map(|ty| (ty.name.as_str(), ty)).collect();

        let mut hash = Keccak256::new();
        for func in &self.funcs {
            hash = hash.update(&hash_func(func, &type_map)?);
        }

        if !self.events.is_empty() {
            let mut ev_hash = Keccak256::new();
            for var in &self.events {
                ev_hash = ev_hash.update(&hash_struct(&var.name, &var.def.fields, &type_map, None)?)
            }
            hash = hash.update(&ev_hash.finalize());
        }

        for s in &self.extends {
            let interface_id = s
                .interface_id
                .ok_or_else(|| format!("service `{}` does not have an `interface_id`", s.name))?;
            hash = hash.update(interface_id.as_bytes());
        }

        Ok(InterfaceId::from_bytes_32(hash.finalize()))
    }
}

fn hash_func(func: &ServiceFunc, type_map: &BTreeMap<&str, &Type>) -> Result<[u8; 32], Error> {
    let mut hash = Keccak256::new();
    hash = match func.kind {
        FunctionKind::Command => hash.update(b"command"),
        FunctionKind::Query => hash.update(b"query"),
    };
    hash = hash.update(func.name.as_bytes());
    for p in &func.params {
        hash = hash.update(&hash_type_decl(&p.type_decl, type_map, None)?);
    }
    hash = hash.update(b"res");
    hash = hash.update(&hash_type_decl(&func.output, type_map, None)?);
    if let Some(th) = &func.throws {
        hash = hash.update(b"throws");
        hash = hash.update(&hash_type_decl(th, type_map, None)?);
    }
    Ok(hash.finalize())
}

fn hash_type(
    ty: &Type,
    type_map: &BTreeMap<&str, &Type>,
    type_params: Option<&BTreeMap<String, TypeDecl>>,
) -> Result<[u8; 32], Error> {
    let bytes = match &ty.def {
        TypeDef::Struct(StructDef { fields }) => {
            hash_struct(&ty.name, fields, type_map, type_params)?
        }
        TypeDef::Enum(enum_def) => {
            let mut hash = Keccak256::new();
            for var in &enum_def.variants {
                hash = hash.update(&hash_struct(
                    &var.name,
                    &var.def.fields,
                    type_map,
                    type_params,
                )?)
            }
            hash.finalize()
        }
        TypeDef::Alias(alias_def) => hash_type_decl(&alias_def.target, type_map, type_params)?,
    };
    Ok(bytes)
}

fn hash_struct(
    name: &str,
    fields: &[StructField],
    type_map: &BTreeMap<&str, &Type>,
    type_params: Option<&BTreeMap<String, TypeDecl>>,
) -> Result<[u8; 32], Error> {
    let mut hash = Keccak256::new().update(name.as_bytes());
    for f in fields {
        hash = hash.update(hash_type_decl(&f.type_decl, type_map, type_params)?.as_slice())
    }
    Ok(hash.finalize())
}

fn hash_type_decl(
    type_decl: &TypeDecl,
    type_map: &BTreeMap<&str, &Type>,
    type_params: Option<&BTreeMap<String, TypeDecl>>,
) -> Result<[u8; 32], Error> {
    let bytes = match type_decl {
        // Encode slices as [T].
        TypeDecl::Slice { item } => Keccak256::new()
            .update(b"[")
            .update(hash_type_decl(item, type_map, type_params)?.as_slice())
            .update(b"]")
            .finalize(),
        // Arrays include the element type and the length.
        TypeDecl::Array { item, len } => Keccak256::new()
            .update(hash_type_decl(item, type_map, type_params)?.as_slice())
            .update(format!("{len}").as_bytes())
            .finalize(),
        // Tuples hash their element types in order.
        TypeDecl::Tuple { types } => {
            let mut hash = Keccak256::new();
            for ty in types {
                hash = hash.update(&hash_type_decl(ty, type_map, type_params)?);
            }
            hash.finalize()
        }
        TypeDecl::Named { name, generics } => {
            // Resolve generic parameters if a mapping is provided (e.g., T -> u32).
            if generics.is_empty()
                && let Some(map) = type_params
                && let Some(param_ty) = map.get(name)
            {
                // generic type parameter `T`
                return hash_type_decl(param_ty, type_map, type_params);
            // Normalize well-known container types to stable markers.
            } else if let Some(ty) = TypeDecl::option_type_decl(type_decl) {
                Keccak256::new()
                    .update(b"Option")
                    .update(&hash_type_decl(&ty, type_map, type_params)?)
                    .finalize()
            } else if let Some((ok, err)) = TypeDecl::result_type_decl(type_decl) {
                Keccak256::new()
                    .update(b"Result")
                    .update(&hash_type_decl(&ok, type_map, type_params)?)
                    .update(&hash_type_decl(&err, type_map, type_params)?)
                    .finalize()
            // Expand named user-defined types from the map, with generics applied.
            } else if let Some(ty) = type_map.get(name.as_str()) {
                if generics.is_empty() {
                    hash_type(ty, type_map, None)?
                } else if ty.type_params.len() == generics.len() {
                    let mut params = BTreeMap::new();
                    for (param, arg) in ty.type_params.iter().zip(generics.iter()) {
                        params.insert(param.name.clone(), arg.clone());
                    }
                    hash_type(ty, type_map, Some(&params))?
                } else {
                    return Err(format!("generic params type `{name}` must be resolved"));
                }
            } else {
                return Err(format!("type `{name}` not supported"));
            }
        }
        // Primitives are hashed by their canonical IDL spelling.
        TypeDecl::Primitive(primitive_type) => Keccak256::new()
            .update(primitive_type.as_str().as_bytes())
            .finalize(),
    };
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use PrimitiveType::*;
    use TypeDecl::*;
    use alloc::{boxed::Box, vec};
    use sails_reflect_hash::ReflectHash;

    macro_rules! assert_type_decl {
        ($ty: ty, $p: expr) => {
            assert_eq!(
                <$ty as ReflectHash>::HASH,
                hash_type_decl(&$p, &BTreeMap::new(), None).unwrap()
            );
        };

        ($ty: ty, $p: expr, $map: expr) => {
            assert_eq!(
                <$ty as ReflectHash>::HASH,
                hash_type_decl(&$p, &$map, None).unwrap()
            );
        };
    }

    #[test]
    fn hash_primitive() {
        assert_type_decl!((), Primitive(Void));
        assert_type_decl!(bool, Primitive(Bool));
        assert_type_decl!(char, Primitive(Char));
        assert_type_decl!(str, Primitive(String));

        assert_type_decl!(u8, Primitive(U8));
        assert_type_decl!(u16, Primitive(U16));
        assert_type_decl!(u32, Primitive(U32));
        assert_type_decl!(u64, Primitive(U64));
        assert_type_decl!(u128, Primitive(U128));

        assert_type_decl!(i8, Primitive(I8));
        assert_type_decl!(i16, Primitive(I16));
        assert_type_decl!(i32, Primitive(I32));
        assert_type_decl!(i64, Primitive(I64));
        assert_type_decl!(i128, Primitive(I128));

        assert_type_decl!(gprimitives::ActorId, Primitive(ActorId));
        assert_type_decl!(gprimitives::CodeId, Primitive(CodeId));
        assert_type_decl!(gprimitives::MessageId, Primitive(MessageId));

        assert_type_decl!(gprimitives::H160, Primitive(H160));
        assert_type_decl!(gprimitives::H256, Primitive(H256));
        assert_type_decl!(gprimitives::U256, Primitive(U256));
    }

    #[test]
    fn hash_slice() {
        assert_type_decl!(
            [u8],
            Slice {
                item: Box::new(Primitive(U8))
            }
        );
        assert_type_decl!(
            Vec<u8>,
            Slice {
                item: Box::new(Primitive(U8))
            }
        );
        assert_type_decl!(
            Vec<(u8, &str)>,
            Slice {
                item: Box::new(Tuple {
                    types: vec![Primitive(U8), Primitive(String)]
                })
            }
        );
    }

    #[test]
    fn hash_array() {
        assert_type_decl!(
            [u8; 32],
            Array {
                item: Box::new(Primitive(U8)),
                len: 32
            }
        );
        assert_type_decl!(
            [(u8, &str); 4],
            Array {
                item: Box::new(Tuple {
                    types: vec![Primitive(U8), Primitive(String)]
                }),
                len: 4
            }
        );
    }

    #[test]
    fn hash_tuple() {
        assert_type_decl!(
            (u8, &str),
            Tuple {
                types: vec![Primitive(U8), Primitive(String)]
            }
        );
        assert_type_decl!(
            (u8, &str, [u8; 32]),
            Tuple {
                types: vec![
                    Primitive(U8),
                    Primitive(String),
                    Array {
                        item: Box::new(Primitive(U8)),
                        len: 32
                    }
                ]
            }
        );
    }

    #[test]
    fn hash_option() {
        assert_type_decl!(
            Option<u8>,
            Named {
                name: "Option".to_string(),
                generics: vec![Primitive(U8)]
            }
        );
        assert_type_decl!(
            Option<(u8, &str, [u8; 32])>,
            Named {
                name: "Option".to_string(),
                generics: vec![Tuple {
                    types: vec![
                        Primitive(U8),
                        Primitive(String),
                        Array {
                            item: Box::new(Primitive(U8)),
                            len: 32
                        }
                    ]
                }]
            }
        );
    }

    #[test]
    fn hash_result() {
        assert_type_decl!(
            Result<u8, &str>,
            Named {
                name: "Result".to_string(),
                generics: vec![Primitive(U8), Primitive(String)]
            }
        );
        assert_type_decl!(
            Result<(u8, &str, [u8; 32]), ()>,
            Named {
                name: "Result".to_string(),
                generics: vec![Tuple {
                    types: vec![
                        Primitive(U8),
                        Primitive(String),
                        Array {
                            item: Box::new(Primitive(U8)),
                            len: 32
                        }
                    ]
                }, Primitive(Void)]
            }
        );
    }

    #[test]
    fn hash_struct_unit() {
        #[derive(ReflectHash)]
        struct UnitStruct;

        let mut map = BTreeMap::new();
        let ty = Type {
            name: "UnitStruct".to_string(),
            type_params: vec![],
            def: TypeDef::Struct(StructDef { fields: vec![] }),
            docs: vec![],
            annotations: vec![],
        };
        map.insert("UnitStruct", &ty);

        assert_type_decl!(
            UnitStruct,
            Named {
                name: "UnitStruct".to_string(),
                generics: vec![]
            },
            map
        );
    }

    #[test]
    fn hash_struct_tuple() {
        #[derive(ReflectHash)]
        #[allow(unused)]
        struct TupleStruct(u32);

        let mut map = BTreeMap::new();
        let ty = Type {
            name: "TupleStruct".to_string(),
            type_params: vec![],
            def: TypeDef::Struct(StructDef {
                fields: vec![StructField {
                    name: None,
                    type_decl: Primitive(U32),
                    docs: vec![],
                    annotations: vec![],
                }],
            }),
            docs: vec![],
            annotations: vec![],
        };
        map.insert("TupleStruct", &ty);

        assert_type_decl!(
            TupleStruct,
            Named {
                name: "TupleStruct".to_string(),
                generics: vec![]
            },
            map
        );
    }

    #[test]
    fn hash_struct_named() {
        #[derive(ReflectHash)]
        #[allow(unused)]
        struct NamedStruct {
            f1: u32,
            f2: Option<&'static str>,
        }

        let mut map = BTreeMap::new();
        let ty = Type {
            name: "NamedStruct".to_string(),
            type_params: vec![],
            def: TypeDef::Struct(StructDef {
                fields: vec![
                    StructField {
                        name: Some("f1".to_string()),
                        type_decl: Primitive(U32),
                        docs: vec![],
                        annotations: vec![],
                    },
                    StructField {
                        name: Some("f2".to_string()),
                        type_decl: Named {
                            name: "Option".to_string(),
                            generics: vec![Primitive(String)],
                        },
                        docs: vec![],
                        annotations: vec![],
                    },
                ],
            }),
            docs: vec![],
            annotations: vec![],
        };
        map.insert("NamedStruct", &ty);

        assert_type_decl!(
            NamedStruct,
            Named {
                name: "NamedStruct".to_string(),
                generics: vec![]
            },
            map
        );
    }

    #[test]
    fn hash_struct_generics() {
        #[derive(ReflectHash)]
        #[allow(unused)]
        struct GenericStruct<T1: ReflectHash, T2: ReflectHash> {
            f1: T1,
            f2: Option<T2>,
        }

        let mut map = BTreeMap::new();
        let ty = Type {
            name: "GenericStruct".to_string(),
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
            def: TypeDef::Struct(StructDef {
                fields: vec![
                    StructField {
                        name: Some("f1".to_string()),
                        type_decl: Named {
                            name: "T1".to_string(),
                            generics: vec![],
                        },
                        docs: vec![],
                        annotations: vec![],
                    },
                    StructField {
                        name: Some("f2".to_string()),
                        type_decl: Named {
                            name: "Option".to_string(),
                            generics: vec![Named {
                                name: "T2".to_string(),
                                generics: vec![],
                            }],
                        },
                        docs: vec![],
                        annotations: vec![],
                    },
                ],
            }),
            docs: vec![],
            annotations: vec![],
        };
        map.insert("GenericStruct", &ty);

        let ty_u8_str = Named {
            name: "GenericStruct".to_string(),
            generics: vec![Primitive(U8), Primitive(String)],
        };
        let ty_str_u8 = Named {
            name: "GenericStruct".to_string(),
            generics: vec![Primitive(String), Primitive(U8)],
        };

        assert_ne!(
            hash_type_decl(&ty_u8_str, &map, None),
            hash_type_decl(&ty_str_u8, &map, None)
        );

        assert_type_decl!(
            GenericStruct<u8, &str>,
            Named {
                name: "GenericStruct".to_string(),
                generics: vec![Primitive(U8), Primitive(String)],
            },
            map
        );

        assert_type_decl!(
            GenericStruct<&str, u8>,
            Named {
                name: "GenericStruct".to_string(),
                generics: vec![Primitive(String), Primitive(U8)],
            },
            map
        );
    }

    #[test]
    fn hash_alias_identical_to_target() {
        let mut map = BTreeMap::new();

        // 1. Define target struct
        let struct_ty = Type {
            name: "TargetStruct".to_string(),
            type_params: vec![],
            def: TypeDef::Struct(StructDef {
                fields: vec![StructField {
                    name: Some("f1".to_string()),
                    type_decl: TypeDecl::Primitive(PrimitiveType::U32),
                    docs: vec![],
                    annotations: vec![],
                }],
            }),
            docs: vec![],
            annotations: vec![],
        };
        map.insert("TargetStruct", &struct_ty);

        // 2. Define alias to that struct
        let alias_ty = Type {
            name: "MyAlias".to_string(),
            type_params: vec![],
            def: TypeDef::Alias(AliasDef {
                target: TypeDecl::Named {
                    name: "TargetStruct".to_string(),
                    generics: vec![],
                },
            }),
            docs: vec![],
            annotations: vec![],
        };
        map.insert("MyAlias", &alias_ty);

        let struct_hash = hash_type_decl(
            &TypeDecl::Named {
                name: "TargetStruct".to_string(),
                generics: vec![],
            },
            &map,
            None,
        )
        .unwrap();

        let alias_hash = hash_type_decl(
            &TypeDecl::Named {
                name: "MyAlias".to_string(),
                generics: vec![],
            },
            &map,
            None,
        )
        .unwrap();

        assert_eq!(struct_hash, alias_hash);
    }
}
