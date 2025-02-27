use crate::export;
use parity_scale_codec::Encode;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::ToTokens;
use std::collections::BTreeMap;
use syn::{
    punctuated::Punctuated, spanned::Spanned, FnArg, GenericArgument, Generics, Ident, ImplItem,
    ImplItemFn, ItemImpl, Lifetime, Pat, Path, PathArguments, PathSegment, ReturnType, Signature,
    Token, Type, TypeImplTrait, TypeParamBound, TypePath, TypeReference, TypeTuple, WhereClause,
};

pub(crate) fn impl_type(item_impl: &ItemImpl) -> (TypePath, PathArguments, Ident) {
    let item_impl_type = item_impl.self_ty.as_ref();
    let path = if let Type::Path(type_path) = item_impl_type {
        type_path.clone()
    } else {
        abort!(
            item_impl_type,
            "failed to parse impl type: {}",
            item_impl_type.to_token_stream()
        )
    };
    let segment = path.path.segments.last().unwrap();
    let args = segment.arguments.clone();
    let ident = segment.ident.clone();
    (path, args, ident)
}

pub(crate) fn impl_type_refs(item_impl_type: &Type) -> (&TypePath, &PathArguments, &Ident) {
    let path = if let Type::Path(type_path) = item_impl_type {
        type_path
    } else {
        abort!(
            item_impl_type,
            "failed to parse impl type: {}",
            item_impl_type.to_token_stream()
        )
    };
    let segment = path.path.segments.last().unwrap();
    let args = &segment.arguments;
    let ident = &segment.ident;
    (path, args, ident)
}

pub(crate) fn impl_constraints(item_impl: &ItemImpl) -> (Generics, Option<WhereClause>) {
    let mut generics = item_impl.generics.clone();
    let where_clause = generics.where_clause.take();
    (generics, where_clause)
}

fn extract_params(handler_signature: &Signature) -> impl Iterator<Item = (&Ident, &Type)> {
    handler_signature.inputs.iter().filter_map(|arg| {
        if let FnArg::Typed(arg) = arg {
            let arg_ident = if let Pat::Ident(arg_ident) = arg.pat.as_ref() {
                &arg_ident.ident
            } else {
                abort!(arg.span(), "unnamed arguments are not supported");
            };
            return Some((arg_ident, arg.ty.as_ref()));
        }
        None
    })
}

pub(crate) fn result_type(handler_signature: &Signature) -> Type {
    match &handler_signature.output {
        ReturnType::Type(_, ty) => *ty.to_owned(),
        ReturnType::Default => Type::Tuple(TypeTuple {
            paren_token: Default::default(),
            elems: Default::default(),
        }),
    }
}

pub(crate) fn unwrap_result_type(handler_signature: &Signature, unwrap_result: bool) -> Type {
    let result_type = result_type(handler_signature);
    // process result type if set unwrap result
    unwrap_result
        .then(|| {
            extract_result_type_from_path(&result_type)
                .unwrap_or_else(|| {
                    abort!(
                        result_type.span(),
                        "`unwrap_result` can be applied to methods returns result only"
                    )
                })
                .clone()
        })
        .unwrap_or(result_type)
}

pub(crate) fn discover_invocation_targets(
    item_impl: &ItemImpl,
    filter: impl Fn(&ImplItemFn) -> bool,
) -> BTreeMap<String, (&ImplItemFn, usize, bool)> {
    item_impl
        .items
        .iter()
        .enumerate()
        .filter_map(|item| {
            if let ImplItem::Fn(fn_item) = item.1 {
                if filter(fn_item) {
                    let (span, route, unwrap_result) = export::invocation_export(fn_item);
                    return Some(((span, route), (fn_item, item.0, unwrap_result)));
                }
            }
            None
        })
        .fold(BTreeMap::new(), |mut result, (route, target)| {
            if let Some(duplicate) = result.insert(route.1, target) {
                abort!(
                    route.0,
                    "`export` or `route` attribute conflicts with one already assigned to '{}'",
                    duplicate.0.sig.ident.to_string()
                );
            }
            result
        })
}

pub(crate) fn extract_lifetime_names(path_args: &PathArguments) -> Vec<String> {
    if let PathArguments::AngleBracketed(type_args) = path_args {
        type_args
            .args
            .iter()
            .filter_map(|a| {
                if let GenericArgument::Lifetime(lifetime) = a {
                    Some(lifetime.ident.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::<String>::new()
    }
}

pub(crate) fn extract_lifetimes(
    path_args: &PathArguments,
) -> Option<impl Iterator<Item = &Lifetime>> {
    if let PathArguments::AngleBracketed(type_args) = path_args {
        Some(type_args.args.iter().filter_map(|a| {
            if let GenericArgument::Lifetime(lifetime) = a {
                Some(lifetime)
            } else {
                None
            }
        }))
    } else {
        None
    }
}

pub(crate) fn replace_any_lifetime_with_static(ty: Type) -> Type {
    match ty {
        Type::Reference(r) => {
            if r.lifetime.is_some() {
                Type::Reference(TypeReference {
                    and_token: r.and_token,
                    lifetime: Some(Lifetime::new("'static", Span::call_site())),
                    mutability: r.mutability,
                    elem: r.elem,
                })
            } else {
                Type::Reference(r)
            }
        }
        Type::Path(p) => Type::Path(TypePath {
            path: replace_lifetime_with_static_in_path(p.path),
            qself: p.qself,
        }),
        _ => ty,
    }
}

fn replace_lifetime_with_static_in_path(path: Path) -> Path {
    let mut segments: Punctuated<PathSegment, Token![::]> = Punctuated::new();
    for s in path.segments {
        segments.push(PathSegment {
            ident: s.ident,
            arguments: replace_lifetime_with_static_in_path_args(s.arguments),
        });
    }
    Path {
        leading_colon: path.leading_colon,
        segments,
    }
}

fn replace_lifetime_with_static_in_path_args(path_args: PathArguments) -> PathArguments {
    if let PathArguments::AngleBracketed(mut type_args) = path_args {
        type_args.args.iter_mut().for_each(|a| match a {
            GenericArgument::Lifetime(lifetime) => {
                *lifetime = Lifetime::new("'static", Span::call_site());
            }
            GenericArgument::Type(ty) => *ty = replace_any_lifetime_with_static(ty.clone()),
            _ => {}
        });
        PathArguments::AngleBracketed(type_args)
    } else {
        path_args
    }
}

#[allow(unused)]
pub(crate) fn remove_lifetimes(path: &Path) -> Path {
    let mut segments: Punctuated<PathSegment, Token![::]> = Punctuated::new();
    for s in &path.segments {
        segments.push(PathSegment {
            ident: s.ident.clone(),
            arguments: PathArguments::None,
        });
    }
    Path {
        leading_colon: path.leading_colon,
        segments,
    }
}

/// Check if type is `CommandReply<T>` and extract inner type `T`
pub(crate) fn extract_reply_type_with_value(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(tp) => extract_reply_result_type(tp),
        Type::ImplTrait(imp) => extract_reply_result_type_from_impl_into(imp),
        _ => None,
    }
}

/// Extract `T` type from `CommandReply<T>`
fn extract_reply_result_type(tp: &TypePath) -> Option<&Type> {
    if let Some(last) = tp.path.segments.last() {
        if last.ident != "CommandReply" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &last.arguments {
            if args.args.len() == 1 {
                if let Some(GenericArgument::Type(ty)) = args.args.first() {
                    return Some(ty);
                }
            }
        }
    }
    None
}

/// Extract `T` type from `impl Into<CommandReply<T>>`
fn extract_reply_result_type_from_impl_into(tit: &TypeImplTrait) -> Option<&Type> {
    if let Some(TypeParamBound::Trait(tr)) = tit.bounds.first() {
        if let Some(last) = tr.path.segments.last() {
            if last.ident != "Into" {
                return None;
            }
            if let PathArguments::AngleBracketed(args) = &last.arguments {
                if args.args.len() == 1 {
                    if let Some(GenericArgument::Type(Type::Path(tp))) = args.args.first() {
                        return extract_reply_result_type(tp);
                    }
                }
            }
        }
    }
    None
}

/// Check if type is `Result<T, E>` and extract inner type `T`
pub(crate) fn extract_result_type_from_path(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => extract_result_type(tp),
        _ => None,
    }
}

/// Extract `T` type from `Result<T, E>`
pub(crate) fn extract_result_type(tp: &TypePath) -> Option<&Type> {
    if let Some(last) = tp.path.segments.last() {
        if last.ident != "Result" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &last.arguments {
            if args.args.len() == 2 {
                if let Some(GenericArgument::Type(ty)) = args.args.first() {
                    return Some(ty);
                }
            }
        }
    }
    None
}

/// Represents parts of a handler function.
#[derive(Clone)]
pub(crate) struct FnBuilder<'a> {
    pub route: String,
    pub encoded_route: Vec<u8>,
    pub impl_fn: &'a ImplItemFn,
    pub ident: &'a Ident,
    pub params_struct_ident: Ident,
    pub params: Vec<(&'a Ident, &'a Type)>,
    pub result_type: Type,
    pub unwrap_result: bool,
    pub sails_path: &'a Path,
}

impl<'a> FnBuilder<'a> {
    pub(crate) fn from(
        route: String,
        impl_fn: &'a ImplItemFn,
        unwrap_result: bool,
        sails_path: &'a Path,
    ) -> Self {
        let encoded_route = route.encode();
        let signature = &impl_fn.sig;
        let ident = &signature.ident;
        let params_struct_ident = Ident::new(&format!("__{}Params", route), Span::call_site());
        let params = extract_params(signature).collect();
        let result_type = unwrap_result_type(signature, unwrap_result);

        Self {
            route,
            encoded_route,
            impl_fn,
            ident,
            params_struct_ident,
            params,
            result_type,
            unwrap_result,
            sails_path,
        }
    }

    pub(crate) fn is_async(&self) -> bool {
        self.impl_fn.sig.asyncness.is_some()
    }

    pub(crate) fn is_query(&self) -> bool {
        self.impl_fn
            .sig
            .receiver()
            .is_none_or(|r| r.mutability.is_none())
    }

    pub(crate) fn result_type_with_value(&self) -> (&Type, bool) {
        let result_type = &self.result_type;
        let (result_type, reply_with_value) = extract_reply_type_with_value(result_type)
            .map_or_else(|| (result_type, false), |ty| (ty, true));

        if reply_with_value && self.is_query() {
            abort!(
                self.result_type.span(),
                "using `CommandReply` type in a query is not allowed"
            );
        }
        (result_type, reply_with_value)
    }
}
