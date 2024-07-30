use crate::route;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use std::collections::BTreeMap;
use syn::{
    punctuated::Punctuated, spanned::Spanned, FnArg, GenericArgument, Ident, ImplItem, ImplItemFn,
    ItemImpl, Lifetime, Pat, Path, PathArguments, PathSegment, Receiver, ReturnType, Signature,
    Token, Type, TypePath, TypeReference, TypeTuple, WhereClause,
};

pub(crate) fn impl_type(item_impl: &ItemImpl) -> (TypePath, PathArguments) {
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
    let args = path.path.segments.last().unwrap().arguments.clone();
    (path, args)
}

pub(crate) fn impl_constraints(item_impl: &ItemImpl) -> Option<WhereClause> {
    item_impl.generics.where_clause.clone()
}

/// Represents parts of a handler function.
#[derive(Clone)]
pub(crate) struct Func<'a> {
    ident: &'a Ident,
    receiver: Option<&'a Receiver>,
    params: Vec<(&'a Ident, &'a Type)>,
    result: Type,
    is_async: bool,
}

impl<'a> Func<'a> {
    pub(crate) fn from(handler_signature: &'a Signature) -> Self {
        let func = &handler_signature.ident;
        let receiver = handler_signature.receiver();
        let params = Self::extract_params(handler_signature).collect();
        let result = Self::extract_result(handler_signature);
        Self {
            ident: func,
            receiver,
            params,
            result,
            is_async: handler_signature.asyncness.is_some(),
        }
    }

    pub(crate) fn ident(&self) -> &Ident {
        self.ident
    }

    pub(crate) fn receiver(&self) -> Option<&Receiver> {
        self.receiver
    }

    pub(crate) fn params(&self) -> &[(&Ident, &Type)] {
        &self.params
    }

    pub(crate) fn result(&self) -> &Type {
        &self.result
    }

    pub(crate) fn is_async(&self) -> bool {
        self.is_async
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

    fn extract_result(handler_signature: &Signature) -> Type {
        result_type(handler_signature)
    }
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

pub(crate) fn discover_invocation_targets(
    item_impl: &ItemImpl,
    filter: impl Fn(&ImplItemFn) -> bool,
) -> BTreeMap<String, (&ImplItemFn, usize)> {
    item_impl
        .items
        .iter()
        .enumerate()
        .filter_map(|item| {
            if let ImplItem::Fn(fn_item) = item.1 {
                if filter(fn_item) {
                    let route = route::invocation_route(fn_item);
                    return Some((route, (fn_item, item.0)));
                }
            }
            None
        })
        .fold(BTreeMap::new(), |mut result, (route, target)| {
            if let Some(duplicate) = result.insert(route.1, target) {
                abort!(
                    route.0,
                    "`route` attribute conflicts with one assigned to '{}'",
                    duplicate.0.sig.ident.to_string()
                );
            }
            result
        })
}

pub(crate) fn generate_unexpected_input_panic(
    input_ident: &Ident,
    message: &str,
    sails_path: &Path,
) -> TokenStream2 {
    let message_pattern = message.to_owned() + ": {}";
    let copy_ident = Ident::new(&format!("__{}", input_ident), Span::call_site());
    quote!({
        let mut #copy_ident = #input_ident;
        let input = String::decode(&mut #copy_ident)
            .unwrap_or_else(|_| {
                if #input_ident.len() <= 8 {
                    format!("0x{}", #sails_path::hex::encode(#input_ident))
                } else {
                    format!(
                        "0x{}..{}",
                        #sails_path::hex::encode(&#input_ident[..4]),
                        #sails_path::hex::encode(&#input_ident[#input_ident.len() - 4..]))
                }
            });
        panic!(#message_pattern, input)
    })
}

pub(crate) fn extract_lifetime_names(path_args: &PathArguments) -> Vec<String> {
    if let PathArguments::AngleBracketed(type_args) = path_args.clone() {
        type_args
            .args
            .into_iter()
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
