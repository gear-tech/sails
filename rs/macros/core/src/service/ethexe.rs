use super::*;
use crate::{
    service::HandlerGenerator,
    shared::{self, Func},
};
use parity_scale_codec::Encode;
use proc_macro2::TokenStream;
use quote::quote;

pub fn service_signature_impl(service_impl: &ItemImpl, sails_path: &Path) -> TokenStream {
    let (service_type_path, _, _) = shared::impl_type_refs(service_impl.self_ty.as_ref());
    let (generics, service_type_constraints) = shared::impl_constraints(service_impl);
    let service_handlers = shared::discover_invocation_targets(service_impl, |fn_item| {
        matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some()
    });
    let service_method_routes =
        service_handlers
            .iter()
            .map(|(handler_route, (handler_fn, _, _))| {
                shared::ethexe::handler_signature(handler_route, handler_fn, sails_path)
            });

    quote! {
        impl #generics #sails_path::solidity::ServiceSignature for #service_type_path #service_type_constraints {
            const METHODS: &'static [#sails_path::solidity::MethodRoute] = &[
                #( #service_method_routes )*
            ];
        }
    }
}

pub fn try_handle_impl(service_impl: &ItemImpl, sails_path: &Path) -> TokenStream {
    let service_handlers = shared::discover_invocation_targets(service_impl, |fn_item| {
        matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some()
    });
    let service_method_branches =
        service_handlers
            .iter()
            .map(|(handler_route, (handler_fn, _, unwrap_result))| {
                try_handle_branch_impl(handler_route, handler_fn, *unwrap_result, sails_path)
            });

    quote! {
        pub async fn try_handle_solidity(
            &mut self,
            method: &[u8],
            input: &[u8],
        ) -> Option<(Vec<u8>, u128)> {
            #( #service_method_branches )*
            None
        }
    }
}

/// Generates code
/// ```rust
/// if method == &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] {
///     // invocation
/// }
/// ```
fn try_handle_branch_impl(
    handler_route: &str,
    handler_fn: &ImplItemFn,
    unwrap_result: bool,
    sails_path: &Path,
) -> TokenStream {
    let handler_route_bytes = handler_route.encode();
    let handler_func = Func::from(&handler_fn.sig);
    let handler_generator = HandlerGenerator::from(handler_func.clone(), unwrap_result);
    let invocation = handler_generator.invocation_func_solidity(sails_path);

    quote! {
        if method == &[ #(#handler_route_bytes),* ] {
            #invocation
        }
    }
}
