use super::*;
use crate::shared;
use convert_case::{Boundary, Case, Casing};
use parity_scale_codec::Encode;
use proc_macro2::TokenStream;
use quote::quote;

/// Generates code
/// ```rust
/// impl sails_rs::solidity::ProgramSignature for MyProgram {
///     const CTORS: &'static [sails_rs::solidity::MethodRoute] = &[(
///         sails_rs::concatcp!(
///             "default", << (u128) as sails_rs::alloy_sol_types::SolValue > ::SolType
///             as sails_rs::alloy_sol_types::SolType > ::SOL_NAME,
///         ),
///         &[28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8] as &[u8],
///     ),
/// ];
/// const SERVICES: &'static [(
///     &'static str,
///     &'static [u8],
///     &'static [sails_rs::solidity::MethodRoute],
/// )] = &[
///     (
///         "service",
///         &[28u8, 83u8, 101u8, 114u8, 118u8, 105u8, 99u8, 101u8] as &[u8],
///         <MyService as sails_rs::solidity::ServiceSignature>::METHODS,
///     ),
/// ];
/// const METHODS_LEN: usize = <MyService as sails_rs::solidity::ServiceSignature>::METHODS
///     .len();
/// }
/// ```
pub fn program_signature_impl(program_impl: &ItemImpl, sails_path: &Path) -> TokenStream {
    let (program_type_path, _, _) = shared::impl_type_refs(program_impl.self_ty.as_ref());
    let (generics, program_type_constraints) = shared::impl_constraints(program_impl);

    let program_ctors = discover_program_ctors(program_impl);
    let program_ctor_sigs = program_ctors
        .iter()
        .map(|(handler_route, (handler_fn, _, _))| {
            shared::ethexe::handler_signature(handler_route, handler_fn, sails_path)
        });

    let services_ctors = discover_services_ctors(program_impl);
    let service_signatures =
        services_ctors
            .iter()
            .map(|(service_route, (service_fn, _, unwrap_result))| {
                service_signature(service_route, service_fn, *unwrap_result, sails_path)
            });

    let methods_len_iter = services_ctors
        .iter()
        .map(|(_, (service_fn, _, unwrap_result))| {
            let service_type = shared::unwrap_result_type(&service_fn.sig, *unwrap_result);
            quote!(<#service_type as #sails_path::solidity::ServiceSignature>::METHODS.len())
        });
    let methods_len = if services_ctors.is_empty() {
        quote! {0}
    } else {
        quote! {#(#methods_len_iter) + *}
    };
    quote! {
        impl #generics #sails_path::solidity::ProgramSignature for #program_type_path #program_type_constraints {
            const CTORS: &'static [#sails_path::solidity::MethodRoute] = &[
                #( #program_ctor_sigs )*
            ];
            const SERVICES: &'static [(&'static str, &'static [u8], &'static [#sails_path::solidity::MethodRoute])] = &[
                #( #service_signatures )*
            ];
            const METHODS_LEN: usize = #methods_len;
        }
    }
}

fn service_signature(
    service_route: &str,
    service_fn: &ImplItemFn,
    unwrap_result: bool,
    sails_path: &Path,
) -> TokenStream {
    let service_route_bytes = service_route.encode();
    let service_name = service_route
        .with_boundaries(&[Boundary::DigitUpper, Boundary::UpperDigit])
        .to_case(Case::Snake);
    let service_type = shared::unwrap_result_type(&service_fn.sig, unwrap_result);

    quote! {
        (
            #service_name,
            &[ #(#service_route_bytes),* ] as &[u8],
            <#service_type as #sails_path::solidity::ServiceSignature>::METHODS,
        ),
    }
}
