use super::*;
use std::collections::BTreeSet;

pub(crate) fn resolve_generic_type_decl(
    type_decl: &TypeDecl,
    type_name: &str,
    type_params: &[sails_idl_meta::TypeParameter],
) -> Result<(TypeDecl, BTreeSet<String>)> {
    let (generic_decl, suffixes) = syn_resolver::try_resolve(type_name, type_decl, type_params)
        .ok_or_else(|| {
            Error::TypeIsUnsupported(format!(
                "Generic type {type_name} not resolved from decl {type_decl}"
            ))
        })?;

    Ok((generic_decl, suffixes))
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
        type_params: &[sails_idl_meta::TypeParameter],
    ) -> Option<(TypeDecl, BTreeSet<String>)> {
        let syn_type = syn::parse_str::<Type>(type_name).ok()?;
        let mut suffixes = BTreeSet::new();
        syn_resolve(&syn_type, type_decl, type_params, &mut suffixes).map(|td| (td, suffixes))
    }

    fn syn_resolve(
        ty: &Type,
        td: &TypeDecl,
        type_params: &[sails_idl_meta::TypeParameter],
        suffixes: &mut BTreeSet<String>,
    ) -> Option<TypeDecl> {
        use TypeDecl::*;

        match (ty, td) {
            (
                Type::Array(TypeArray {
                    elem,
                    len: len_expr,
                    ..
                }),
                Array { item, len },
            ) => {
                // const generic arguments support limited to Array type
                let len = if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(lit_int),
                    ..
                }) = len_expr
                    && let Ok(len) = lit_int.base10_parse::<u32>()
                {
                    len
                } else {
                    let len_str = len_expr.to_token_stream().to_string();
                    suffixes.insert(format!("{len_str}{len}"));
                    *len
                };
                let item = syn_resolve(elem, item, type_params, suffixes)?;
                Some(Array {
                    item: Box::new(item),
                    len,
                })
            }
            (Type::Slice(TypeSlice { elem, .. }), Slice { item }) => {
                let item = syn_resolve(elem, item, type_params, suffixes)?;
                Some(Slice {
                    item: Box::new(item),
                })
            }
            (Type::Tuple(TypeTuple { elems, .. }), Tuple { types })
                if elems.len() == types.len() =>
            {
                let types: Option<Vec<TypeDecl>> = elems
                    .iter()
                    .zip(types)
                    .map(|(ty, td)| syn_resolve(ty, td, type_params, suffixes))
                    .collect();
                Some(Tuple { types: types? })
            }
            (Type::Reference(TypeReference { elem, .. }), _) => {
                syn_resolve(elem, td, type_params, suffixes)
            }
            // No paren types in the final output. Only single value tuples
            (Type::Paren(TypeParen { elem, .. }), _) => {
                syn_resolve(elem, td, type_params, suffixes)
            }
            (Type::Path(TypePath { path, .. }), type_decl) => {
                if let Some(generic) = generic_param(type_params, path) {
                    return Some(generic);
                }

                match type_decl {
                    Slice { item } => {
                        let last_segment = path.segments.last()?;
                        let ty_name = last_segment.ident.to_string();
                        let mut ty_generics: Vec<&Type> = Vec::new();
                        if let PathArguments::AngleBracketed(syn_args) = &last_segment.arguments {
                            for arg in &syn_args.args {
                                if let GenericArgument::Type(t) = arg {
                                    ty_generics.push(t)
                                }
                            }
                        }
                        if ty_name == "Vec"
                            && let [elem] = ty_generics.as_slice()
                        {
                            let item = syn_resolve(elem, item, type_params, suffixes)?;
                            return Some(Slice {
                                item: Box::new(item),
                            });
                        }
                        if ty_name == "BTreeMap"
                            && let Tuple { types } = item.as_ref()
                            && let [key, value] = types.as_slice()
                            && let [ty_key, ty_value] = ty_generics.as_slice()
                        {
                            let key = syn_resolve(ty_key, key, type_params, suffixes)?;
                            let value = syn_resolve(ty_value, value, type_params, suffixes)?;
                            return Some(Slice {
                                item: Box::new(Tuple {
                                    types: vec![key, value],
                                }),
                            });
                        }

                        None
                    }
                    Array { item: _, len: _ } => None,
                    Tuple { types: _ } => None,
                    Named { name, generics } => {
                        let last_segment = path.segments.last()?;
                        // let ty_name = last_segment.ident.to_string();
                        // TODO: const generic arguments are still ignored.
                        // We never add anything to the `suffixes` set when we see `GenericArgument::Const`,
                        // which means `TypeResolver::register_user_defined` (see rs/idl-gen/src/type_resolver.rs:163-207)
                        // cannot differentiate two instantiations of the same type that only differ by const parameters
                        // unless those consts appear inside an array length.
                        // For types such as `struct Tag<const N: usize>;` or wrappers that only forward the const generic to another const generic,
                        // every instantiation (`Tag<1>`, `Tag<2>`, …) collapses to the same IDL name,
                        // so the second definition silently reuses the metadata of the first and the registry becomes inconsistent.
                        let mut ty_generics: Vec<&Type> = Vec::new();
                        if let PathArguments::AngleBracketed(syn_args) = &last_segment.arguments {
                            for arg in &syn_args.args {
                                if let GenericArgument::Type(t) = arg {
                                    ty_generics.push(t)
                                }
                            }
                        }
                        let generics: Option<Vec<TypeDecl>> = ty_generics
                            .iter()
                            .zip(generics)
                            .map(|(ty, td)| syn_resolve(ty, td, type_params, suffixes))
                            .collect();
                        Some(Named {
                            name: name.clone(),
                            generics: generics?,
                        })
                    }
                    Primitive(_) => Some(type_decl.clone()),
                }
            }
            _ => None,
        }
    }

    fn generic_param(
        type_params: &[sails_idl_meta::TypeParameter],
        path: &syn::Path,
    ) -> Option<TypeDecl> {
        if let Some(ident) = path.get_ident()
            && type_params.iter().any(|tp| *ident == tp.name)
        {
            Some(TypeDecl::Named {
                name: ident.to_string(),
                generics: vec![],
            })
        } else {
            None
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
        f1: T,
        f2: Option<T>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct ConstStruct<const N: usize> {
        f1: [u8; N],
        f2: Option<[u8; N]>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct ConstGenericStruct<T, const N: usize> {
        f1: GenericStruct<T>,
        f2: ConstStruct<N>,
    }

    #[test]
    fn generic_resolver_generic_struct() {
        use sails_idl_meta::{PrimitiveType::*, TypeDecl::*};

        let meta_type = MetaType::new::<GenericStruct<u32>>();
        let mut registry = Registry::new();
        let id = registry.register_type(&meta_type).id;
        let portable_registry = PortableRegistry::from(registry);
        let mut resolver = TypeResolver::from_registry(&portable_registry);
        let ty = portable_registry.resolve(id).unwrap();
        let type_params = resolver.resolve_type_params(ty).unwrap();

        let type_decl = resolver.get(id).unwrap();

        let (generic_decl, _) =
            resolve_generic_type_decl(type_decl, "GenericStruct<T>", &type_params).unwrap();

        assert_eq!(
            &Named {
                name: "GenericStruct".to_string(),
                generics: vec![Primitive(U32)]
            },
            type_decl
        );

        assert_eq!(
            Named {
                name: "GenericStruct".to_string(),
                generics: vec![Named {
                    name: "T".to_string(),
                    generics: vec![]
                }]
            },
            generic_decl
        );
    }

    #[test]
    fn generic_resolver_cosnt_struct() {
        use sails_idl_meta::TypeDecl::*;

        let meta_type = MetaType::new::<ConstStruct<32>>();
        let mut registry = Registry::new();
        let id = registry.register_type(&meta_type).id;
        let portable_registry = PortableRegistry::from(registry);
        let mut resolver = TypeResolver::from_registry(&portable_registry);
        let ty = portable_registry.resolve(id).unwrap();
        let type_params = resolver.resolve_type_params(ty).unwrap();

        let type_decl = resolver.get(id).unwrap();

        let (generic_decl, _) =
            resolve_generic_type_decl(type_decl, "ConstStruct<N>", &type_params).unwrap();

        assert_eq!(
            &Named {
                name: "ConstStructN32".to_string(),
                generics: vec![]
            },
            type_decl
        );

        assert_eq!(
            Named {
                name: "ConstStructN32".to_string(),
                generics: vec![]
            },
            generic_decl
        );
    }

    #[test]
    fn generic_resolver_generic_cosnt_struct() {
        use sails_idl_meta::{PrimitiveType::*, TypeDecl::*};

        let meta_type_u8_32 = MetaType::new::<ConstGenericStruct<u8, 32>>();
        let meta_type_u8_64 = MetaType::new::<ConstGenericStruct<u8, 64>>();
        let mut registry = Registry::new();
        let u8_32_id = registry.register_type(&meta_type_u8_32).id;
        let _u8_64_id = registry.register_type(&meta_type_u8_64).id;
        let portable_registry = PortableRegistry::from(registry);
        let mut resolver = TypeResolver::from_registry(&portable_registry);
        let ty = portable_registry.resolve(u8_32_id).unwrap();
        let type_params = resolver.resolve_type_params(ty).unwrap();

        let type_decl = resolver.get(u8_32_id).unwrap();

        let (generic_decl, _) =
            resolve_generic_type_decl(type_decl, "ConstGenericStruct<T, N>", &type_params).unwrap();

        assert_eq!(
            &Named {
                name: "ConstGenericStruct".to_string(),
                generics: vec![Primitive(U8)]
            },
            type_decl
        );

        assert_eq!(
            Named {
                name: "ConstGenericStruct".to_string(),
                generics: vec![Named {
                    name: "T".to_string(),
                    generics: vec![]
                }]
            },
            generic_decl
        );
    }
}
