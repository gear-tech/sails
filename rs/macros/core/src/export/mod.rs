use args::ExportArgs;
use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use syn::{parse::Parse, spanned::Spanned, Attribute, ImplItemFn};

use crate::{route, shared};

mod args;

pub fn export(attrs: TokenStream, impl_item_fn_tokens: TokenStream) -> TokenStream {
    let fn_impl: ImplItemFn = syn::parse2::<ImplItemFn>(impl_item_fn_tokens.clone())
        .unwrap_or_else(|err| {
            abort!(
                err.span(),
                "`export` attribute can be applied to methods only: {}",
                err
            )
        });
    ensure_pub_visibility(&fn_impl);
    ensure_single_export_or_route_on_impl(&fn_impl);
    let args = syn::parse2::<ExportArgs>(attrs)
        .unwrap_or_else(|_| abort!(fn_impl.span(), "`export` attribute cannot be parsed"));
    ensure_returns_result_with_unwrap_result(fn_impl, args);
    impl_item_fn_tokens
}

fn ensure_pub_visibility(fn_impl: &ImplItemFn) {
    match fn_impl.vis {
        syn::Visibility::Public(_) => (),
        _ => abort!(
            fn_impl.span(),
            "`export` attribute can be applied to public methods only"
        ),
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
            "multiple `export` or `route` attributes on the same method are not allowed",
        )
    }
}

fn ensure_returns_result_with_unwrap_result(fn_impl: ImplItemFn, args: ExportArgs) {
    // ensure Result type is returned if unwrap_result is set to true
    _ = shared::unwrap_result_type(&fn_impl.sig, args.unwrap_result());
}

pub(crate) fn invocation_export(fn_impl: &ImplItemFn) -> (Span, String, bool) {
    if let Some((args, span)) = parse_export_args(&fn_impl.attrs) {
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

fn parse_export_args(attrs: &[Attribute]) -> Option<(ExportArgs, Span)> {
    attrs
        .iter()
        .filter_map(|attr| parse_attr(attr).map(|args| (args, attr.meta.span())))
        .next()
}

pub(crate) fn parse_attr(attr: &Attribute) -> Option<ExportArgs> {
    let meta = attr.meta.require_list().ok()?;
    if meta
        .path
        .segments
        .last()
        .is_some_and(|s| s.ident == "export")
    {
        let args = meta
            .parse_args_with(ExportArgs::parse)
            .unwrap_or_else(|er| {
                abort!(meta.span(), "`export` attribute cannot be parsed: {}", er)
            });
        Some(args)
    } else {
        None
    }
}
