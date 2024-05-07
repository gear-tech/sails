use crate::route;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use std::collections::BTreeMap;
use syn::{
    spanned::Spanned, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Pat, PathArguments, Receiver,
    ReturnType, Signature, Type, TypePath, TypeTuple, WhereClause,
};

/// A struct that represents the type of an `impl` block.
pub(crate) struct ImplType<'a> {
    path: &'a TypePath,
    args: &'a PathArguments,
    constraints: Option<&'a WhereClause>,
}

impl<'a> ImplType<'a> {
    pub(crate) fn new(item_impl: &'a ItemImpl) -> Self {
        let path = {
            let item_impl_type = item_impl.self_ty.as_ref();
            if let Type::Path(type_path) = item_impl_type {
                type_path
            } else {
                abort!(
                    item_impl_type.span(),
                    "Failed to parse impl type: {}",
                    item_impl_type.to_token_stream()
                )
            }
        };
        let args = &path.path.segments.last().unwrap().arguments;
        let constraints = item_impl.generics.where_clause.as_ref();
        Self {
            path,
            args,
            constraints,
        }
    }

    pub(crate) fn path(&self) -> &TypePath {
        self.path
    }

    pub(crate) fn args(&self) -> &PathArguments {
        self.args
    }

    pub(crate) fn constraints(&self) -> Option<&WhereClause> {
        self.constraints
    }
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
                    abort!(arg.span(), "Unnamed arguments are not supported");
                };
                return Some((arg_ident, arg.ty.as_ref()));
            }
            None
        })
    }

    fn extract_result(handler_signature: &Signature) -> Type {
        match &handler_signature.output {
            ReturnType::Type(_, ty) => *ty.to_owned(),
            ReturnType::Default => Type::Tuple(TypeTuple {
                paren_token: Default::default(),
                elems: Default::default(),
            }),
        }
    }
}

pub(crate) fn discover_invocation_targets(
    item_impl: &ItemImpl,
    filter: impl Fn(&ImplItemFn) -> bool,
    allow_empty_route: bool, // Even though we always pass `false` here, we keep this parameter for the case when we want to allow anonymously exposed services
) -> BTreeMap<String, &Signature> {
    item_impl
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Fn(fn_item) = item {
                if filter(fn_item) {
                    let route = route::invocation_route(fn_item);
                    if route.1.is_empty() && !allow_empty_route {
                        abort!(route.0, "Empty route is not allowed")
                    }
                    return Some((route, &fn_item.sig));
                }
            }
            None
        })
        .fold(BTreeMap::new(), |mut result, (route, target)| {
            if let Some(duplicate) = result.insert(route.1, target) {
                abort!(
                    route.0,
                    "Route conflicts with one assigned to '{}'",
                    duplicate.ident.to_string()
                );
            }
            result
        })
}

pub(crate) fn generate_unexpected_input_panic(input_ident: &Ident, message: &str) -> TokenStream2 {
    let message_pattern = message.to_owned() + ": {}";
    quote!({
        let input = String::decode(&mut #input_ident)
            .unwrap_or_else(|_| {
                if #input_ident.len() <= 8 {
                    format!("0x{}", hex::encode(#input_ident))
                } else {
                    format!(
                        "0x{}..{}",
                        hex::encode(&#input_ident[..4]),
                        hex::encode(&#input_ident[#input_ident.len() - 4..]))
                }
            });
        panic!(#message_pattern, input)
    })
}
