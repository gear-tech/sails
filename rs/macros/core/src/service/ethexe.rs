use crate::shared::{self, Func};
use convert_case::{Case, Casing};
use parity_scale_codec::Encode;
use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

pub fn service_signature_impl(service_impl: &ItemImpl, sails_path: &Path) -> TokenStream {
    let (service_type_path, _, _) = shared::impl_type_refs(&service_impl);
    let (generics, service_type_constraints) = shared::impl_constraints(&service_impl);
    let service_handlers = shared::discover_invocation_targets(service_impl, |fn_item| {
        matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some()
    });
    let service_method_routes =
        service_handlers
            .into_iter()
            .map(|(handler_route, (handler_fn, _, _))| {
                handler_signature(handler_route, handler_fn, sails_path)
            });

    quote! {
        impl #generics #sails_path::solidity::ServiceSignature for #service_type_path #service_type_constraints {
            const METHODS: &'static [#sails_path::solidity::MethodRoute] = &[
                #( #service_method_routes )*
            ];
        }
    }
}

fn handler_signature(
    handler_route: String,
    handler_fn: &ImplItemFn,
    sails_path: &Path,
) -> TokenStream {
    let handler_route_bytes = handler_route.encode();
    let handler_name = handler_route.to_case(Case::Snake);
    let handler_func = Func::from(&handler_fn.sig);
    let handler_types = handler_func.params().iter().map(|item| {
        let param_type = item.1;
        quote!(#param_type,)
    });

    quote! {
        (
            #sails_path::concatcp!(
                #handler_name,
                <<(#(#handler_types)*) as #sails_path::alloy_sol_types::SolValue>::SolType as #sails_path::alloy_sol_types::SolType>::SOL_NAME,
            ),
            &[ #(#handler_route_bytes),* ] as &[u8],
        ),
    }
}
