use crate::export;
use convert_case::{Case, Casing};
use parity_scale_codec::Encode;
use proc_macro_error::abort;
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::BTreeMap;
use syn::{
    FnArg, GenericArgument, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, Lifetime, Pat, Path,
    PathArguments, PathSegment, ReturnType, Signature, Token, Type, TypeImplTrait, TypeParamBound,
    TypePath, TypeReference, TypeTuple, WhereClause, punctuated::Punctuated, spanned::Spanned,
};

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
    if unwrap_result {
        {
            extract_result_type_from_path(&result_type)
                .unwrap_or_else(|| {
                    abort!(
                        result_type.span(),
                        "`unwrap_result` can be applied to methods returns result only"
                    )
                })
                .clone()
        }
    } else {
        result_type
    }
}

pub(crate) fn invocation_export(fn_impl: &ImplItemFn) -> Option<(Span, String, bool, bool)> {
    export::parse_export_args(&fn_impl.attrs).map(|(args, span)| {
        let ident = &fn_impl.sig.ident;
        let unwrap_result = args.unwrap_result();
        let route = args.route().map_or_else(
            || ident.to_string().to_case(Case::Pascal),
            |route| route.to_case(Case::Pascal),
        );
        (span, route, unwrap_result, true)
    })
}

pub(crate) fn invocation_export_or_default(fn_impl: &ImplItemFn) -> (Span, String, bool, bool) {
    invocation_export(fn_impl).unwrap_or_else(|| {
        let ident = &fn_impl.sig.ident;
        (
            ident.span(),
            ident.to_string().to_case(Case::Pascal),
            false,
            false,
        )
    })
}

pub(crate) fn discover_invocation_targets<'a>(
    item_impl: &'a ItemImpl,
    filter: impl Fn(&ImplItemFn) -> bool,
    sails_path: &'a Path,
) -> Vec<FnBuilder<'a>> {
    let mut routes = BTreeMap::<String, String>::new();
    let mut vec: Vec<FnBuilder<'a>> = item_impl
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Fn(fn_item) = item {
                if filter(fn_item) {
                    let (span, route, unwrap_result, export) =
                        invocation_export_or_default(fn_item);

                    if let Some(duplicate) =
                        routes.insert(route.clone(), fn_item.sig.ident.to_string())
                    {
                        abort!(
                            span,
                            "`export` attribute conflicts with one already assigned to '{}'",
                            duplicate
                        );
                    }
                    let fn_builder =
                        FnBuilder::from(route, export, fn_item, unwrap_result, sails_path);
                    return Some(fn_builder);
                }
            }
            None
        })
        .collect();
    vec.sort_by(|a, b| a.route.cmp(&b.route));
    vec
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
    pub export: bool,
    pub encoded_route: Vec<u8>,
    pub impl_fn: &'a ImplItemFn,
    pub ident: &'a Ident,
    pub params_struct_ident: Ident,
    params_idents: Vec<&'a Ident>,
    params_types: Vec<&'a Type>,
    pub result_type: Type,
    pub unwrap_result: bool,
    pub sails_path: &'a Path,
}

impl<'a> FnBuilder<'a> {
    pub(crate) fn from(
        route: String,
        export: bool,
        impl_fn: &'a ImplItemFn,
        unwrap_result: bool,
        sails_path: &'a Path,
    ) -> Self {
        let encoded_route = route.encode();
        let signature = &impl_fn.sig;
        let ident = &signature.ident;
        let params_struct_ident = Ident::new(&format!("__{route}Params"), Span::call_site());
        let (params_idents, params_types): (Vec<_>, Vec<_>) = extract_params(signature).unzip();
        let result_type = unwrap_result_type(signature, unwrap_result);

        Self {
            route,
            export,
            encoded_route,
            impl_fn,
            ident,
            params_struct_ident,
            params_idents,
            params_types,
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

    pub(crate) fn params(&self) -> impl Iterator<Item = (&&Ident, &&Type)> {
        self.params_idents.iter().zip(self.params_types.iter())
    }

    pub(crate) fn params_idents(&self) -> &[&Ident] {
        self.params_idents.as_slice()
    }

    #[cfg(feature = "ethexe")]
    pub(crate) fn params_types(&self) -> &[&Type] {
        self.params_types.as_slice()
    }

    #[cfg(feature = "ethexe")]
    pub(crate) fn route_camel_case(&self) -> String {
        use convert_case::{Boundary, Case, Casing};

        self.route
            .with_boundaries(&[Boundary::UNDERSCORE, Boundary::LOWER_UPPER])
            .to_case(Case::Camel)
    }
}
