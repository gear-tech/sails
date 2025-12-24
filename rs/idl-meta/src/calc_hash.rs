use super::*;
use alloc::collections::btree_map::BTreeMap;
use keccak_const::Keccak256;

// macro_rules! hash {
//     ($b: ident, $expr: expr) => {
//         $b = $b.update($expr);
//     };
// }

impl ServiceUnit {
    pub fn inteface_id(&self) -> InterfaceId {
        let type_hashes: BTreeMap<_, _> = self
            .types
            .iter()
            .map(|ty| (ty.name.clone(), hash_type(ty)))
            .collect();

        let mut hash = Keccak256::new();
        for func in &self.funcs {
            hash = hash.update(&func_hash(func));
        }

        if !self.events.is_empty() {
            let mut ev_hash = Keccak256::new();
            for var in &self.events {
                ev_hash = ev_hash.update(&hash_struct(&var.name, &var.def.fields))
            }
            hash = hash.update(&ev_hash.finalize());
        }

        for s in &self.extends {
            // TODO
            let interface_id = s.interface_id.unwrap();
            hash = hash.update(interface_id.as_bytes());
        }

        InterfaceId::from_bytes_32(hash.finalize())
    }
}

fn func_hash(func: &ServiceFunc) -> [u8; 32] {
    let mut hash = Keccak256::new();
    hash = match func.kind {
        FunctionKind::Command => hash.update(b"command"),
        FunctionKind::Query => hash.update(b"query"),
    };
    hash = hash.update(func.name.as_bytes());
    for p in &func.params {
        hash = hash.update(&hash_type_decl(&p.type_decl));
    }
    hash = hash.update(b"res");
    hash = hash.update(&hash_type_decl(&func.output));
    if let Some(th) = &func.throws {
        hash = hash.update(b"res");
        hash = hash.update(&hash_type_decl(th));
    }
    hash.finalize()
}

fn hash_type(ty: &Type) -> [u8; 32] {
    match &ty.def {
        TypeDef::Struct(StructDef { fields }) => hash_struct(&ty.name, &fields),
        TypeDef::Enum(enum_def) => {
            let mut hash = Keccak256::new();
            for var in &enum_def.variants {
                hash = hash.update(&hash_struct(&var.name, &var.def.fields))
            }
            hash.finalize()
        }
    }
}

fn hash_struct(name: &str, fields: &[StructField]) -> [u8; 32] {
    let mut hash = Keccak256::new().update(name.as_bytes());
    for f in fields {
        hash = hash.update(hash_type_decl(&f.type_decl).as_slice())
    }
    hash.finalize()
}

fn hash_type_decl(type_decl: &TypeDecl) -> [u8; 32] {
    match type_decl {
        TypeDecl::Slice { item } => Keccak256::new()
            .update(b"[")
            .update(hash_type_decl(item).as_slice())
            .update(b"]")
            .finalize(),
        TypeDecl::Array { item, len } => Keccak256::new()
            .update(hash_type_decl(item).as_slice())
            .update(format!("{len}").as_bytes())
            .finalize(),
        TypeDecl::Tuple { types } => {
            let mut hash = Keccak256::new();
            for ty in types {
                hash = hash.update(&hash_type_decl(ty));
            }
            hash.finalize()
        }
        TypeDecl::Named { name, generics } => {
            if let Some(ty) = TypeDecl::option_type_decl(type_decl) {
                Keccak256::new()
                    .update(b"Option")
                    .update(&hash_type_decl(&ty))
                    .finalize()
            } else if let Some((ok, err)) = TypeDecl::result_type_decl(type_decl) {
                Keccak256::new()
                    .update(b"Result")
                    .update(&hash_type_decl(&ok))
                    .update(&hash_type_decl(&err))
                    .finalize()
            } else {
                [0; 32]
            }
        }
        TypeDecl::Primitive(primitive_type) => Keccak256::new()
            .update(primitive_type.as_str().as_bytes())
            .finalize(),
    }
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
            assert_eq!(<$ty as ReflectHash>::HASH, hash_type_decl(&$p));
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
}
