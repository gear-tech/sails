use convert_case::{Case, Casing};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use syn::{spanned::Spanned, ImplItemFn, Lit};

pub fn groute(_attrs: TokenStream2, impl_item_fn_tokens: TokenStream2) -> TokenStream2 {
    impl_item_fn_tokens
}

pub(crate) fn invocation_route(invocation_func: &ImplItemFn) -> String {
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
                Some((span, lit.value()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if routes.len() > 1 {
        abort!(
            routes[1].0,
            "Multiple groute attributes are not allowed for the same service function"
        );
    }
    routes
        .first()
        .map(|(_, route)| route)
        .unwrap_or_else(|| &service_func_ident)
        .to_case(Case::Pascal)
}
