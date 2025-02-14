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

pub fn program_const(program_impl: &ItemImpl, sails_path: &Path) -> TokenStream {
    let (program_type_path, _, _) = shared::impl_type_refs(program_impl.self_ty.as_ref());
    quote! {
        const __CTOR_SIGS: [[u8; 4]; <#program_type_path as #sails_path::solidity::ProgramSignature>::CTORS.len()]
            = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::ctor_sigs();
        const __METHOD_SIGS: [[u8; 4]; <#program_type_path as #sails_path::solidity::ProgramSignature>::METHODS_LEN]
            = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::method_sigs();
        const __METHOD_ROUTES: [(&'static [u8], &'static [u8]); <#program_type_path as #sails_path::solidity::ProgramSignature>::METHODS_LEN]
            = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::method_routes();
    }
}

pub fn match_ctor_impl(program_impl: &ItemImpl, sails_path: &Path) -> TokenStream {
    let (program_type_path, _, _) = shared::impl_type_refs(program_impl.self_ty.as_ref());
    let program_ctors = discover_program_ctors(program_impl);
    let ctor_branches =
        program_ctors
            .iter()
            .map(|(handler_route, (handler_fn, _, unwrap_result))| {
                ctor_branch_impl(
                    program_type_path,
                    handler_route,
                    handler_fn,
                    *unwrap_result,
                    sails_path,
                )
            });
    quote! {
        async fn match_ctor_solidity(ctor: &[u8], input: &[u8]) -> Option<#program_type_path> {
            #( #ctor_branches )*
            None
        }
    }
}

fn ctor_branch_impl(
    program_type_path: &TypePath,
    ctor_route: &str,
    ctor_fn: &ImplItemFn,
    unwrap_result: bool,
    sails_path: &Path,
) -> TokenStream {
    let handler_route_bytes = ctor_route.encode();
    let handler_func = Func::from(&ctor_fn.sig);
    let handler_ident = handler_func.ident();
    let handler_params = handler_func.params().iter().map(|item| {
        let param_ident = item.0;
        quote!(#param_ident)
    });
    let handler_params_comma = handler_func.params().iter().map(|item| {
        let param_ident = item.0;
        quote!(#param_ident,)
    });
    let handler_types = handler_func.params().iter().map(|item| {
        let param_type = item.1;
        quote!(#param_type,)
    });

    let await_token = handler_func.is_async().then(|| quote!(.await));
    let unwrap_token = unwrap_result.then(|| quote!(.unwrap()));

    quote! {
        if ctor == &[ #(#handler_route_bytes),* ] {
            let (#(#handler_params_comma)*) : (#(#handler_types)*) = #sails_path::alloy_sol_types::SolValue::abi_decode_params(input, false).expect("Failed to decode request");
            let program = #program_type_path :: #handler_ident (#(#handler_params),*) #await_token #unwrap_token;
            return Some(program);
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
