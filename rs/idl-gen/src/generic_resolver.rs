use super::*;
use std::collections::HashSet;

pub(crate) fn resolve_generic_type_decl(
    type_decl: &TypeDecl,
    type_name: &str,
    type_params: &Vec<sails_idl_meta::TypeParameter>,
) -> TypeDecl {
    let candidates = build_generic_candidates(type_decl, type_params);

    println!(
        "type_decl: {:?}, type_name: {}, candidates: {:?}",
        type_decl.to_string(),
        type_name,
        candidates
            .iter()
            .map(|td| td.to_string())
            .collect::<Vec<_>>()
    );
    candidates
        .into_iter()
        .find(|td| td.to_string() == type_name)
        .unwrap_or_else(|| panic!("Not Resolved {}", type_name))
    // TODO: Resolve with type_params
    // generics::resolve(type_name)
}

struct GenericCandidates<'a> {
    resolved: HashSet<TypeDecl>,
    type_params: Vec<(&'a TypeDecl, &'a str)>,
}

impl<'a> GenericCandidates<'a> {
    fn new(type_params: &'a Vec<sails_idl_meta::TypeParameter>) -> Self {
        Self {
            resolved: HashSet::new(),
            type_params: type_params
                .iter()
                .filter_map(|tp| tp.ty.as_ref().map(|ty| (ty, tp.name.as_str())))
                .collect(),
        }
    }

    fn try_push(&mut self, candidate: TypeDecl, f: impl Fn(TypeDecl) -> TypeDecl) {
        for (td, name) in &self.type_params {
            if *td == &candidate {
                self.resolved.insert(f(generic_type_decl(name)));
            }
        }
        self.resolved.insert(f(candidate));
    }
}

fn build_generic_candidates(
    type_decl: &TypeDecl,
    type_params: &Vec<sails_idl_meta::TypeParameter>,
) -> HashSet<TypeDecl> {
    let mut candidates = GenericCandidates::new(type_params);
    // try push to candidates `type_decl` as generic param
    candidates.try_push(type_decl.clone(), |td| td);
    match type_decl {
        TypeDecl::Slice(item) => {
            let decls = build_generic_candidates(item, type_params);
            for item in decls {
                candidates.try_push(item, |td| TypeDecl::Slice(Box::new(td)));
            }
        }
        TypeDecl::Array { item, len } => {
            let decls = build_generic_candidates(item, type_params);
            for item in decls {
                candidates.try_push(item, |td| TypeDecl::Array {
                    item: Box::new(td),
                    len: *len,
                });
            }
        }
        TypeDecl::Tuple(type_decls) => {
            for (idx, item) in type_decls.iter().enumerate() {
                let decls = build_generic_candidates(item, type_params);
                let type_decls_resolved: Vec<_> = candidates
                    .resolved
                    .iter()
                    .filter_map(|td| match td {
                        TypeDecl::Tuple(decls) => Some(decls.clone()),
                        _ => None,
                    })
                    .collect();
                for tds in type_decls_resolved {
                    for item in &decls {
                        candidates.try_push(item.clone(), |td| {
                            let mut tds = tds.clone();
                            tds[idx] = td;
                            TypeDecl::Tuple(tds)
                        });
                    }
                }
            }
        }
        TypeDecl::Option(item) => {
            let decls = build_generic_candidates(item, type_params);
            for item in decls {
                candidates.try_push(item, |td| TypeDecl::Option(Box::new(td)));
            }
        }
        TypeDecl::Result { ok, err } => {
            let decls = build_generic_candidates(ok, type_params);
            for item in decls {
                candidates.try_push(item, |td| TypeDecl::Result {
                    ok: Box::new(td),
                    err: err.clone(),
                });
            }
            let decls = build_generic_candidates(err, type_params);
            for item in decls {
                candidates.try_push(item, |td| TypeDecl::Result {
                    ok: ok.clone(),
                    err: Box::new(td),
                });
            }
        }
        TypeDecl::Primitive(_) => {
            // already pushed as `type_decl`
        }
        TypeDecl::UserDefined { name, generics } => {
            for (idx, item) in generics.iter().enumerate() {
                let decls = build_generic_candidates(item, type_params);
                let type_decls_resolved: Vec<_> = candidates
                    .resolved
                    .iter()
                    .filter_map(|td| match td {
                        TypeDecl::UserDefined { name: _, generics } => Some(generics.clone()),
                        _ => None,
                    })
                    .collect();

                for tds in type_decls_resolved {
                    for item in &decls {
                        candidates.try_push(item.clone(), |td| {
                            let mut tds = tds.clone();
                            tds[idx] = td;
                            TypeDecl::UserDefined {
                                name: name.to_string(),
                                generics: tds,
                            }
                        });
                    }
                }
            }
        }
        TypeDecl::Generic(_) => {}
    };
    candidates.resolved
}

fn generic_type_decl(name: &str) -> TypeDecl {
    TypeDecl::Generic(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::{MetaType, PortableRegistry, Registry, TypeInfo};
    use std::collections::BTreeMap;

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct GenericStruct<T> {
        field: T,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum GenericEnum<T1, T2> {
        Variant1(T1),
        Variant2(T2),
        Variant3(T1, Option<T2>),
        Variant4(Option<(T1, GenericStruct<T2>, u32)>),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    pub enum ManyVariants {
        One,
        Two(u32),
        Three(Option<Vec<gprimitives::U256>>),
        Four { a: u32, b: Option<u16> },
        Five(String, Vec<u8>),
        Six((u32,)),
        Seven(GenericEnum<u32, String>),
        Eight([BTreeMap<u32, String>; 10]),
        Nine(TupleVariantsDocs),
    }

    #[derive(TypeInfo)]
    pub enum TupleVariantsDocs {
        /// Docs for no tuple docs 1
        NoTupleDocs1(u32, String),
        NoTupleDocs2(gprimitives::CodeId, Vec<u8>),
        /// Docs for tuple docs 1
        TupleDocs1(
            u32,
            /// This is the second field
            String,
        ),
        TupleDocs2(
            /// This is the first field
            u32,
            /// This is the second field
            String,
        ),
        /// Docs for struct docs
        StructDocs {
            /// This is field `a`
            a: u32,
            /// This is field `b`
            b: String,
        },
    }

    #[test]
    fn generic_resolver_struct_primitive() {
        use PrimitiveType::*;
        use TypeDecl::*;

        let meta_type = MetaType::new::<GenericStruct<u32>>();
        let mut registry = Registry::new();
        let id = registry.register_type(&meta_type).id;
        let portable_registry = PortableRegistry::from(registry);
        let mut resolver = TypeResolver::from_registry(&portable_registry);
        let ty = portable_registry.resolve(id).unwrap();
        let type_params = resolver.resolve_type_params(ty);

        let type_decl = resolver.get(id).unwrap();

        let candidates = build_generic_candidates(type_decl, &type_params);
        println!("{:?}", candidates);

        assert_eq!(2, candidates.len());
        assert!(candidates.contains(&UserDefined {
            name: "GenericStruct".to_string(),
            generics: vec![Primitive(U32)]
        }));
        assert!(candidates.contains(&UserDefined {
            name: "GenericStruct".to_string(),
            generics: vec![Generic("T".to_string())]
        }));

        // let string_struct = resolver.get(string_struct_id).unwrap();
        // assert_eq!(string_struct.to_string(), "GenericStruct<String>");
    }
}
