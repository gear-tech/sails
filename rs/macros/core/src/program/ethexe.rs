use super::*;
use convert_case::{Boundary, Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

impl ProgramBuilder {
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
    pub fn program_signature_impl(&self) -> TokenStream {
        let sails_path = self.sails_path();
        let (program_type_path, _, _) = self.impl_type();
        let (generics, program_type_constraints) = self.impl_constraints();

        let program_ctors = self.program_ctors();
        let program_ctor_sigs = program_ctors
            .iter()
            .map(|fn_builder| fn_builder.sol_handler_signature());

        let service_ctors = self.service_ctors();
        let service_ctor_sigs = service_ctors
            .iter()
            .map(|fn_builder| fn_builder.sol_service_signature());

        let methods_len_iter = service_ctors.iter().map(|fn_builder| {
            let service_type = &fn_builder.result_type;
            quote!(<#service_type as #sails_path::solidity::ServiceSignature>::METHODS.len())
        });
        let methods_len = if service_ctors.is_empty() {
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
                    #( #service_ctor_sigs )*
                ];
                const METHODS_LEN: usize = #methods_len;
            }
        }
    }

    pub fn program_const(&self) -> TokenStream {
        let sails_path = self.sails_path();
        let (program_type_path, _, _) = self.impl_type();

        quote! {
            const __CTOR_SIGS: [[u8; 4]; <#program_type_path as #sails_path::solidity::ProgramSignature>::CTORS.len()]
                = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::ctor_sigs();
            const __METHOD_SIGS: [[u8; 4]; <#program_type_path as #sails_path::solidity::ProgramSignature>::METHODS_LEN]
                = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::method_sigs();
            const __METHOD_ROUTES: [(&'static [u8], &'static [u8]); <#program_type_path as #sails_path::solidity::ProgramSignature>::METHODS_LEN]
                = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::method_routes();
        }
    }

    pub fn match_ctor_impl(&self) -> TokenStream {
        let (program_type_path, _, _) = self.impl_type();
        let program_ctors = self.program_ctors();
        let ctor_branches = program_ctors
            .iter()
            .map(|fn_builder| fn_builder.ctor_branch_impl(program_type_path));
        quote! {
            async fn match_ctor_solidity(ctor: &[u8], input: &[u8]) -> Option<#program_type_path> {
                #( #ctor_branches )*
                None
            }
        }
    }
}

impl FnBuilder<'_> {
    fn sol_service_signature(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let service_route_bytes = self.encoded_route.as_slice();
        let service_name = self
            .route
            .with_boundaries(&[Boundary::DigitUpper, Boundary::UpperDigit])
            .to_case(Case::Snake);
        let service_type = &self.result_type;

        quote! {
            (
                #service_name,
                &[ #(#service_route_bytes),* ] as &[u8],
                <#service_type as #sails_path::solidity::ServiceSignature>::METHODS,
            ),
        }
    }

    pub(crate) fn sol_handler_signature(&self) -> TokenStream {
        use convert_case::{Case, Casing};

        let sails_path = self.sails_path;
        let handler_route_bytes = self.encoded_route.as_slice();
        let handler_name = self.route.to_case(Case::Snake);
        let handler_types = self
            .params
            .iter()
            .map(|item| {
                let param_type = item.1;
                quote!(#param_type,)
            })
            .chain([quote!(u128,)]); // add uint128 to method signature

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

    fn ctor_branch_impl(&self, program_type_path: &TypePath) -> TokenStream {
        let sails_path = self.sails_path;
        let handler_route_bytes = self.encoded_route.as_slice();
        let handler_ident = self.ident;
        let handler_params = self.params.iter().map(|item| {
            let param_ident = item.0;
            quote!(#param_ident)
        });
        let handler_params_comma = self.params.iter().map(|item| {
            let param_ident = item.0;
            quote!(#param_ident,)
        });
        let handler_types = self.params.iter().map(|item| {
            let param_type = item.1;
            quote!(#param_type,)
        });

        let await_token = self.is_async.then(|| quote!(.await));
        let unwrap_token = self.unwrap_result.then(|| quote!(.unwrap()));

        quote! {
            if ctor == &[ #(#handler_route_bytes),* ] {
                let (#(#handler_params_comma)*) : (#(#handler_types)*) = #sails_path::alloy_sol_types::SolValue::abi_decode_params(input, false).expect("Failed to decode request");
                let program = #program_type_path :: #handler_ident (#(#handler_params),*) #await_token #unwrap_token;
                return Some(program);
            }
        }
    }
}
