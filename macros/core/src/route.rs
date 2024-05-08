use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use syn::{spanned::Spanned, Ident, ImplItemFn, Lit};

pub fn groute(_attrs: TokenStream, impl_item_fn_tokens: TokenStream) -> TokenStream {
    let service_impl: ImplItemFn = syn::parse2::<ImplItemFn>(impl_item_fn_tokens.clone())
        .unwrap_or_else(|err| abort!(err.span(), "Failed to parse function impl: {}", err));
    match service_impl.vis {
        syn::Visibility::Public(_) => impl_item_fn_tokens,
        _ => abort!(
            service_impl.span(),
            "Function impl with route must be public"
        ),
    }
}

pub(crate) fn invocation_route(invocation_func: &ImplItemFn) -> (Span, String) {
    let service_func_ident = invocation_func.sig.ident.to_string();
    let routes = invocation_func
        .attrs
        .iter()
        .filter_map(|attr| attr.meta.require_list().ok())
        .filter(|attr| {
            attr.path
                .segments
                .last()
                .map(|s| s.ident == "groute")
                .unwrap_or(false)
        })
        .filter_map(|attr| {
            syn::parse2::<Lit>(attr.tokens.clone())
                .map(|lit| (attr.span(), lit))
                .ok()
        })
        .filter_map(|(span, lit)| {
            if let Lit::Str(lit) = lit {
                let route = lit.value();
                _ = syn::parse_str::<Ident>(&route).map_err(|err| {
                    abort!(
                        lit.span(),
                        "Route name must be a literal with a valid Rust identifier: {}",
                        err
                    )
                });
                Some((span, route))
            } else {
                abort!(
                    lit.span(),
                    "Route name must be a literal with a valid Rust identifier",
                )
            }
        })
        .collect::<Vec<_>>();
    if routes.len() > 1 {
        abort!(
            routes[1].0,
            "Multiple groute attributes are not allowed for the same function"
        );
    }
    routes
        .first()
        .map(|(span, route)| (*span, route.to_case(Case::Pascal)))
        .unwrap_or_else(|| {
            (
                invocation_func.sig.ident.span(),
                service_func_ident.to_case(Case::Pascal),
            )
        })
}
