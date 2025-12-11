use super::*;
use std::collections::HashSet;

pub(crate) fn resolve_generic_type_decl(
    type_decl: &TypeDecl,
    type_name: &str,
    type_params: &Vec<sails_idl_meta::TypeParameter>,
) -> Result<(TypeDecl, Vec<String>)> {
    let (generic_decl, suffixes) =
        syn_resolver::try_resolve(type_name, type_decl).ok_or_else(|| {
            Error::TypeIsUnsupported(format!(
                "Generic type {type_name} not resolved from decl {type_decl}"
            ))
        })?;
    let match_name = generic_decl.to_string();
    let candidates = build_generic_candidates(type_decl, type_params);

    println!(
        "type_decl: {:?}, type_name: {}, match_name: {}, suffixes: {:?}, candidates: {:?}",
        type_decl.to_string(),
        type_name,
        match_name,
        &suffixes,
        candidates
            .iter()
            .map(|td| td.to_string())
            .collect::<Vec<_>>()
    );
    candidates
        .into_iter()
        .find(|td| td.to_string() == match_name)
        .ok_or_else(|| {
            Error::TypeIsUnsupported(format!(
                "Generic type {type_name} not resolved from decl {type_decl}"
            ))
        })
        .map(|td| (td, suffixes))
}

struct GenericCandidates<'a> {
    resolved: HashSet<TypeDecl>,
    type_params: Vec<(&'a TypeDecl, &'a str)>,
}

impl<'a> GenericCandidates<'a> {
    fn new(type_params: &'a [sails_idl_meta::TypeParameter]) -> Self {
        Self {
            resolved: HashSet::new(),
            type_params: type_params
                .iter()
                .filter_map(|tp| tp.ty.as_ref().map(|ty| (ty, tp.name.as_str())))
                .collect(),
        }
    }

    fn push(&mut self, candidate: TypeDecl, f: impl Fn(TypeDecl) -> TypeDecl) {
        for &(td, name) in &self.type_params {
            if td == &candidate {
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
    // push `type_decl` as generic param to candidates
    candidates.push(type_decl.clone(), |td| td);
    match type_decl {
        TypeDecl::Slice { item } => {
            let decls = build_generic_candidates(item, type_params);
            for item in decls {
                candidates.push(item, |td| TypeDecl::Slice { item: Box::new(td) });
            }
        }
        TypeDecl::Array { item, len } => {
            let decls = build_generic_candidates(item, type_params);
            for item in decls {
                candidates.push(item, |td| TypeDecl::Array {
                    item: Box::new(td),
                    len: *len,
                });
            }
        }
        TypeDecl::Tuple { types } => {
            for (idx, item) in types.iter().enumerate() {
                let decls = build_generic_candidates(item, type_params);
                let type_decls_resolved: Vec<_> = candidates
                    .resolved
                    .iter()
                    .filter_map(|td| match td {
                        TypeDecl::Tuple { types } => Some(types.clone()),
                        _ => None,
                    })
                    .collect();
                for tds in type_decls_resolved {
                    for item in &decls {
                        candidates.push(item.clone(), |td| {
                            let mut types = tds.clone();
                            types[idx] = td;
                            TypeDecl::Tuple { types }
                        });
                    }
                }
            }
        }
        TypeDecl::Primitive(_) => {
            // already pushed as `type_decl`
        }
        TypeDecl::Named { name, generics } => {
            for (idx, item) in generics.iter().enumerate() {
                let decls = build_generic_candidates(item, type_params);
                let type_decls_resolved: Vec<_> = candidates
                    .resolved
                    .iter()
                    .filter_map(|td| match td {
                        TypeDecl::Named {
                            name: resolved_name,
                            generics,
                        } if resolved_name == name => Some(generics.clone()),
                        _ => None,
                    })
                    .collect();

                for tds in type_decls_resolved {
                    for item in &decls {
                        candidates.push(item.clone(), |td| {
                            let mut generics = tds.clone();
                            generics[idx] = td;
                            TypeDecl::Named {
                                name: name.to_string(),
                                generics,
                            }
                        });
                    }
                }
            }
        }
    };
    candidates.resolved
}

fn generic_type_decl(name: &str) -> TypeDecl {
    TypeDecl::named(name.to_string())
}

mod syn_resolver {
    use super::*;
    use quote::ToTokens;
    use syn::{
        GenericArgument, PathArguments, Type, TypeArray, TypeParen, TypePath, TypeReference,
        TypeSlice, TypeTuple,
    };

    pub(super) fn try_resolve(
        type_name: &str,
        type_decl: &TypeDecl,
    ) -> Option<(TypeDecl, Vec<String>)> {
        let syn_type = syn::parse_str::<Type>(type_name).ok()?;
        let mut suffixes = Vec::new();
        syn_resolve(&syn_type, type_decl, &mut suffixes).map(|td| (td, suffixes))
    }

    fn syn_resolve(t: &Type, type_decl: &TypeDecl, suffixes: &mut Vec<String>) -> Option<TypeDecl> {
        use TypeDecl::*;

        match t {
            Type::Array(TypeArray { elem, len, .. }) => {
                let len_str = len.to_token_stream().to_string();
                let len = if let Ok(len) = len_str.parse::<u32>() {
                    len
                } else if let TypeDecl::Array { item: _, len } = type_decl {
                    suffixes.push(format!("{len_str}{len}"));
                    *len
                } else {
                    return None;
                };
                let item = syn_resolve(elem, type_decl, suffixes)?;
                Some(Array {
                    item: Box::new(item),
                    len,
                })
            }
            Type::Slice(TypeSlice { elem, .. }) => Some(Slice {
                item: Box::new(syn_resolve(elem, type_decl, suffixes)?),
            }),
            Type::Tuple(TypeTuple { elems, .. }) => Some(Tuple {
                types: elems
                    .iter()
                    .filter_map(|t| syn_resolve(t, type_decl, suffixes))
                    .collect(),
            }),
            Type::Reference(TypeReference { elem, .. }) => syn_resolve(elem, type_decl, suffixes),
            // No paren types in the final output. Only single value tuples
            Type::Paren(TypeParen { elem, .. }) => syn_resolve(elem, type_decl, suffixes),
            Type::Path(TypePath { path, .. }) => {
                let last_segment = path.segments.last()?;
                let name = last_segment.ident.to_string();

                let generics: Vec<_> =
                    if let PathArguments::AngleBracketed(syn_args) = &last_segment.arguments {
                        syn_args
                            .args
                            .iter()
                            .filter_map(|arg| match arg {
                                GenericArgument::Type(t) => syn_resolve(t, type_decl, suffixes),
                                _ => None,
                            })
                            .collect()
                    } else {
                        vec![]
                    };
                match name.as_str() {
                    "Vec" => {
                        if let [td] = generics.as_slice() {
                            Some(Slice {
                                item: Box::new(td.clone()),
                            })
                        } else {
                            Some(Named { name, generics })
                        }
                    }
                    "BTreeMap" => {
                        if let [_, _] = generics.as_slice() {
                            Some(Slice {
                                item: Box::new(Tuple { types: generics }),
                            })
                        } else {
                            Some(Named { name, generics })
                        }
                    }
                    _ => Some(Named { name, generics }),
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_resolver::TypeResolver;
    use scale_info::{MetaType, PortableRegistry, Registry, TypeInfo};

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct GenericStruct<T> {
        field: T,
    }

    #[test]
    fn generic_resolver_struct_primitive() {
        use sails_idl_meta::{PrimitiveType::*, TypeDecl::*};

        let meta_type = MetaType::new::<GenericStruct<u32>>();
        let mut registry = Registry::new();
        let id = registry.register_type(&meta_type).id;
        let portable_registry = PortableRegistry::from(registry);
        let mut resolver = TypeResolver::from_registry(&portable_registry);
        let ty = portable_registry.resolve(id).unwrap();
        let type_params = resolver.resolve_type_params(ty).unwrap();

        let type_decl = resolver.get(id).unwrap();

        let candidates = build_generic_candidates(type_decl, &type_params);

        assert_eq!(2, candidates.len());
        assert!(candidates.contains(&Named {
            name: "GenericStruct".to_string(),
            generics: vec![Primitive(U32)]
        }));
        assert!(candidates.contains(&Named {
            name: "GenericStruct".to_string(),
            generics: vec![Named {
                name: "T".to_string(),
                generics: vec![]
            }]
        }));
    }
}
