use args::ExportArgs;
use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use syn::{spanned::Spanned, ImplItemFn};

use crate::{route, shared};

mod args;

pub fn export(attrs: TokenStream, impl_item_fn_tokens: TokenStream) -> TokenStream {
    let fn_impl: ImplItemFn = syn::parse2::<ImplItemFn>(impl_item_fn_tokens.clone())
        .unwrap_or_else(|err| {
            abort!(
                err.span(),
                "`export` attribute can be applied to impls only: {}",
                err
            )
        });
    ensure_pub_visibility(&fn_impl);
    ensure_single_export_or_route_on_impl(&fn_impl);
    let args = syn::parse2::<ExportArgs>(attrs)
        .unwrap_or_else(|_| abort!(fn_impl.span(), "`export` attribute cannot be parsed"));
    ensure_return_result_type_if_unwrap_result(&fn_impl, args.unwrap_result());
    impl_item_fn_tokens
}

fn ensure_pub_visibility(fn_impl: &ImplItemFn) {
    match fn_impl.vis {
        syn::Visibility::Public(_) => (),
        _ => abort!(
            fn_impl.span(),
            "`export` attribute can be applied to public impls only"
        ),
    }
}

fn ensure_return_result_type_if_unwrap_result(fn_impl: &ImplItemFn, unwrap_result: bool) {
    let ty = shared::result_type(&fn_impl.sig);
    if unwrap_result && shared::extract_result_type_from_path(&ty).is_none() {
        abort!(
            fn_impl.span(),
            "`export` attribute with `unwrap_result` can only be applied to impls returning `Result<T, E>`"
        )
    }
}

pub(crate) fn ensure_single_export_or_route_on_impl(fn_impl: &ImplItemFn) {
    let attr_export = fn_impl.attrs.iter().find(|attr| {
        attr.meta
            .path()
            .segments
            .last()
            .map(|s| s.ident == "export" || s.ident == "route")
            .unwrap_or(false)
    });
    if attr_export.is_some() {
        abort!(
            fn_impl,
            "multiple `export` or `route` attributes on the same impl are not allowed",
        )
    }
}

pub(crate) fn invocation_export(fn_impl: &ImplItemFn) -> (Span, String, bool) {
    let parsed_args = fn_impl
        .attrs
        .iter()
        .filter_map(|attr| attr.meta.require_list().ok())
        .find(|meta| {
            meta.path
                .segments
                .last()
                .map(|s| s.ident == "export")
                .unwrap_or(false)
        })
        .map(|meta| {
            let args = syn::parse2::<ExportArgs>(meta.tokens.clone()).unwrap_or_else(|er| {
                abort!(meta.span(), "`export` attribute cannot be parsed: {}", er)
            });
            (args, meta.tokens.span())
        });

    if let Some((args, span)) = parsed_args {
        let ident = &fn_impl.sig.ident;
        let unwrap_result = args.unwrap_result();
        args.route().map_or_else(
            || {
                (
                    ident.span(),
                    ident.to_string().to_case(Case::Pascal),
                    unwrap_result,
                )
            },
            |route| (span, route.to_case(Case::Pascal), unwrap_result),
        )
    } else {
        let (span, route) = route::invocation_route(fn_impl);
        (span, route, false)
    }
}
