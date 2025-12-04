use super::*;
use proc_macro2::TokenStream;
use quote::quote;

impl ProgramBuilder {
    /// Generates code
    /// ```rust
    /// impl sails_rs::solidity::ProgramSignature for MyProgram {
    ///     const CTORS: &'static [sails_rs::solidity::MethodExpo] = &[(
    ///         &[24u8, 67u8, 114u8, 101u8, 97u8, 116u8, 101u8] as &[u8],
    ///         "create",
    ///         <<(bool) as sails_rs::alloy_sol_types::SolValue>::SolType as sails_rs::alloy_sol_types::SolType>::SOL_NAME,
    ///         <<() as sails_rs::alloy_sol_types::SolValue>::SolType as sails_rs::alloy_sol_types::SolType>::SOL_NAME,
    ///     ),];
    /// const SERVICES: &'static [sails_rs::solidity::ServiceExpo] = &[
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
            .map(|fn_builder| fn_builder.sol_handler_signature(false));

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
                const CTORS: &'static [#sails_path::solidity::MethodExpo] = &[
                    #( #program_ctor_sigs, )*
                ];
                const SERVICES: &'static [#sails_path::solidity::ServiceExpo] = &[
                    #( #service_ctor_sigs, )*
                ];
                const METHODS_LEN: usize = #methods_len;
            }
        }
    }

    pub fn program_const(&self) -> TokenStream {
        let sails_path = self.sails_path();
        let (program_type_path, ..) = self.impl_type();

        quote! {
            const __CTOR_SIGS: [[u8; 4]; <#program_type_path as #sails_path::solidity::ProgramSignature>::CTORS.len()]
                = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::ctor_sigs();
            const __CTOR_CALLBACK_SIGS: [[u8; 4]; <#program_type_path as #sails_path::solidity::ProgramSignature>::CTORS.len()]
                = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::ctor_callback_sigs();
            const __METHOD_SIGS: [[u8; 4]; <#program_type_path as #sails_path::solidity::ProgramSignature>::METHODS_LEN]
                = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::method_sigs();
            const __METHOD_ROUTES: [(&'static [u8], &'static [u8]); <#program_type_path as #sails_path::solidity::ProgramSignature>::METHODS_LEN]
                = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::method_routes();
            const __CALLBACK_SIGS: [[u8; 4]; <#program_type_path as #sails_path::solidity::ProgramSignature>::METHODS_LEN]
                = #sails_path::solidity::ConstProgramMeta::<#program_type_path>::callback_sigs();
        }
    }

    pub fn match_ctor_impl(&self, program_ident: &Ident) -> TokenStream {
        let (program_type_path, ..) = self.impl_type();
        let program_ctors = self.program_ctors();
        let ctor_branches = program_ctors
            .iter()
            .map(|fn_builder| fn_builder.sol_ctor_branch_impl(program_type_path, program_ident));

        quote! {
            fn match_ctor_solidity(ctor: &[u8], input: &[u8]) -> Option<bool> {
                #( #ctor_branches )*
                None
            }
        }
    }

    pub fn sol_init(&self, input_ident: &Ident) -> TokenStream {
        let sails_path = self.sails_path();
        let (program_type_path, ..) = self.impl_type();

        quote! {
            if let Ok(sig) = TryInto::<[u8; 4]>::try_into(&#input_ident[..4]) {
                if let Some(idx) = __CTOR_SIGS.iter().position(|s| s == &sig) {
                    let (ctor_route, ..) = <#program_type_path as #sails_path::solidity::ProgramSignature>::CTORS[idx];
                    if let Some(encode_reply) = match_ctor_solidity(ctor_route, &#input_ident[4..]) {
                        // add callbak selector if `encode_reply` is set
                        if encode_reply {
                            let output = [__CTOR_CALLBACK_SIGS[idx].as_slice(), gstd::msg::id().into_bytes().as_slice()].concat();
                            gstd::msg::reply_bytes(output, 0).expect("Failed to send output");
                        }
                        return;
                    }
                }
            }
        }
    }

    pub fn sol_main(&self, solidity_dispatchers: &[TokenStream]) -> TokenStream {
        quote! {
            if let Ok(sig) = TryInto::<[u8; 4]>::try_into(&input[..4]) {
                if let Some(idx) = __METHOD_SIGS.iter().position(|s| s == &sig) {
                    let (route, method) = __METHOD_ROUTES[idx];
                    #(#solidity_dispatchers)*
                }
            }
        }
    }
}

impl FnBuilder<'_> {
    fn sol_service_signature(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let service_route_bytes = self.encoded_route.as_slice();
        let service_name = self.route_camel_case();
        let service_type = &self.result_type;

        quote! {
            (
                #service_name,
                &[ #(#service_route_bytes),* ] as &[u8],
                <#service_type as #sails_path::solidity::ServiceSignature>::METHODS,
            )
        }
    }

    pub(crate) fn sol_handler_signature(&self, is_service: bool) -> TokenStream {
        let sails_path = self.sails_path;
        let handler_route_bytes = self.encoded_route.as_slice();
        let handler_name = if is_service {
            // method name as PascalCase
            &self.route
        } else {
            // ctor name as camelCase
            &self.route_camel_case()
        };
        let handler_types = self.params_types();
        let (result_type, _) = self.result_type_with_value();

        // add `bool` to method signature as first parameter as encode reply
        let handler_types = quote! { bool, #(#handler_types,)* };

        // add MessageId (alloy_primitives::B256) to callback signature as first parameter
        let callback_types = if is_service {
            quote! { #sails_path::alloy_primitives::B256, #result_type }
        } else {
            quote! { #sails_path::alloy_primitives::B256, }
        };

        quote! {
            (
                &[ #(#handler_route_bytes),* ] as &[u8],
                #handler_name,
                <<(#handler_types) as #sails_path::alloy_sol_types::SolValue>::SolType as #sails_path::alloy_sol_types::SolType>::SOL_NAME,
                <<(#callback_types) as #sails_path::alloy_sol_types::SolValue>::SolType as #sails_path::alloy_sol_types::SolType>::SOL_NAME,
            )
        }
    }

    fn sol_ctor_branch_impl(
        &self,
        program_type_path: &TypePath,
        program_ident: &Ident,
    ) -> TokenStream {
        let sails_path = self.sails_path;
        let handler_route_bytes = self.encoded_route.as_slice();
        let handler_ident = self.ident;
        let handler_params = self.params_idents();
        let handler_types = self.params_types();

        let unwrap_token = self.unwrap_result.then(|| quote!(.unwrap()));

        let ctor_invocation = if self.is_async() {
            quote! {
                gstd::message_loop(async move {
                    let program = #program_type_path :: #handler_ident (#(#handler_params),*).await #unwrap_token;
                    unsafe { #program_ident = Some(program) };
                });
            }
        } else {
            quote! {
                let program = #program_type_path :: #handler_ident (#(#handler_params),*) #unwrap_token;
                unsafe { #program_ident = Some(program) };
            }
        };

        let payable_check = self.payable_check();

        // read uint128 as first parameter
        quote! {
            if ctor == &[ #(#handler_route_bytes),* ] {
                let (__encode_reply, #(#handler_params,)*) : (bool, #(#handler_types,)*) = #sails_path::alloy_sol_types::SolValue::abi_decode_params(input).expect("Failed to decode request");
                #payable_check
                #ctor_invocation
                return Some(__encode_reply);
            }
        }
    }

    pub(crate) fn sol_service_invocation(&self) -> TokenStream2 {
        let route_ident = &self.route_ident();
        let service_ctor_ident = self.ident;

        quote! {
            if route == & #route_ident {
                let mut service = program_ref.#service_ctor_ident();
                let Some(is_async) = service.check_asyncness(method) else {
                    gstd::unknown_input_panic("Unknown service method", &input);
                };
                if is_async {
                    gstd::message_loop(async move {
                        let (output, value, encode_reply) = service
                            .try_handle_solidity_async(method, &input[4..])
                            .await
                            .unwrap_or_else(|| {
                                gstd::unknown_input_panic("Unknown request", &input)
                            });
                        // add callbak selector if `encode_reply` is set
                        let output = if encode_reply {
                            let selector = __CALLBACK_SIGS[idx];
                            [selector.as_slice(), output.as_slice()].concat()
                        } else {
                            output
                        };
                        gstd::msg::reply_bytes(output, value).expect("Failed to send output");
                    });
                } else {
                    let (output, value, encode_reply) = service
                        .try_handle_solidity(method, &input[4..])
                        .unwrap_or_else(|| {
                            gstd::unknown_input_panic("Unknown request", &input)
                        });
                    // add callbak selector if `encode_reply` is set
                    let output = if encode_reply {
                        let selector = __CALLBACK_SIGS[idx];
                        [selector.as_slice(), output.as_slice()].concat()
                    } else {
                        output
                    };
                    gstd::msg::reply_bytes(output, value).expect("Failed to send output");
                }

                return;
            }
        }
    }
}
