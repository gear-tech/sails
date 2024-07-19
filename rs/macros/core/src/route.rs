use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use syn::{spanned::Spanned, Ident, ImplItemFn, Lit};

pub fn groute(_attrs: TokenStream, impl_item_fn_tokens: TokenStream) -> TokenStream {
    let route_fn_impl: ImplItemFn = syn::parse2::<ImplItemFn>(impl_item_fn_tokens.clone())
        .unwrap_or_else(|err| {
            abort!(
                err.span(),
                "`groute` attribute can be applied to impls only: {}",
                err
            )
        });
    match route_fn_impl.vis {
        syn::Visibility::Public(_) => impl_item_fn_tokens,
        _ => abort!(
            route_fn_impl.span(),
            "`groute` attribute can be applied to public methods only"
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
                        "`groute` attribute requires a literal with a valid Rust identifier: {}",
                        err
                    )
                });
                Some((span, route))
            } else {
                abort!(
                    lit.span(),
                    "`groute` attribute requires a literal with a valid Rust identifier",
                )
            }
        })
        .collect::<Vec<_>>();
    if routes.len() > 1 {
        abort!(
            routes[1].0,
            "multiple `groute` attributes are not allowed for the same method"
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
