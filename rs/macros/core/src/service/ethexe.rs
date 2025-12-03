use super::*;
use proc_macro2::TokenStream;
use quote::quote;

impl ServiceBuilder<'_> {
    pub(super) fn service_signature_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let service_type_path = self.type_path;
        let generics = &self.generics;
        let service_type_constraints = self.type_constraints.as_ref();
        let service_method_expo = self
            .service_handlers
            .iter()
            .map(|fn_builder| fn_builder.sol_handler_signature(true));

        let combined_methods = if self.base_types.is_empty() {
            quote! {
                &[#( #service_method_expo, )*]
            }
        } else {
            let base_methods = self.base_types.iter().map(|path| {
                let path_wo_lifetimes = shared::remove_lifetimes(path);
                quote! {
                    <#path_wo_lifetimes as #sails_path::solidity::ServiceSignature>::METHODS
                }
            });
            quote! {
                #sails_path::const_concat_slices!(
                    <#sails_path::solidity::MethodExpo>,
                    &[#( #service_method_expo, )*],
                    #( #base_methods ),*
                )
            }
        };

        quote! {
            impl #generics #sails_path::solidity::ServiceSignature for #service_type_path #service_type_constraints {
                const METHODS: &'static [#sails_path::solidity::MethodExpo] = #combined_methods;
            }
        }
    }

    pub(super) fn try_handle_solidity_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let inner_ident = &self.inner_ident;

        let impl_inner = |is_async: bool| {
            let (name_ident, asyncness, await_token) = if is_async {
                (
                    quote!(try_handle_solidity_async),
                    Some(quote!(async)),
                    Some(quote!(.await)),
                )
            } else {
                (quote!(try_handle_solidity), None, None)
            };

            let service_method_branches = self.service_handlers.iter().filter_map(|fn_builder| {
                if is_async == fn_builder.is_async() {
                    Some(fn_builder.sol_try_handle_branch_impl())
                } else {
                    None
                }
            });

            let base_invocation = if self.base_types.is_empty() {
                None
            } else {
                let base_types = self.base_types;
                let base_exposure_invocations = base_types.iter().enumerate().map(|(idx, _)| {
                    let idx_token = if base_types.len() == 1 { None } else {
                        let idx_literal = Literal::usize_unsuffixed(idx);
                        Some(quote! { . #idx_literal })
                    };
                    quote! {
                        if let Some(result) = base_services #idx_token .expose(self.route) . #name_ident(method, input) #await_token {
                            return Some(result);
                        }
                    }
                });
                // Base Services, as `Into` tuple from Service
                Some(quote! {
                    let base_services: ( #( #base_types ),* ) = self. #inner_ident .into();
                    #( #base_exposure_invocations )*
                })
            };

            quote! {
                pub #asyncness fn #name_ident(
                    mut self,
                    method: &[u8],
                    input: &[u8],
                ) -> Option<(#sails_path::Vec<u8>, u128, bool)> {
                    use #sails_path::gstd::services::{Service, Exposure};

                    #( #service_method_branches )*
                    #base_invocation
                    None
                }
            }
        };

        let sync_impl = impl_inner(false);
        let async_impl = impl_inner(true);

        quote! {
            #sync_impl
            #async_impl
        }
    }

    pub(super) fn exposure_emit_eth_impls(&self) -> Option<TokenStream> {
        let sails_path = self.sails_path;

        self.events_type.map(|events_type| {
            quote! {
                pub fn emit_eth_event(&self, event: #events_type) -> #sails_path::errors::Result<()> {
                    use #sails_path::gstd::services::ExposureWithEvents;

                    self.emitter().emit_eth_event(event)
                }
            }
        })
    }
}

impl FnBuilder<'_> {
    /// Generates code
    /// ```rust
    /// if method == &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] {
    ///     // invocation
    /// }
    /// ```
    fn sol_try_handle_branch_impl(&self) -> TokenStream {
        let handler_route_bytes = self.encoded_route.as_slice();
        let invocation = self.sol_invocation_func();

        quote! {
            if method == &[ #(#handler_route_bytes),* ] {
                #invocation
            }
        }
    }

    /// Generates code for encode/decode parameters and fn invocation
    /// ```rust
    /// let (__encode_reply, p1, p2): (bool, u32, String) = sails_rs::alloy_sol_types::SolValue::abi_decode_params(input, false).ok()?;
    /// let result: u32 = self.do_this(p1, p2).await;
    /// let value = 0u128;
    /// ```
    fn sol_invocation_func(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let handler_ident = self.ident;
        let handler_params = self.params_idents();
        let handler_types = self.params_types();

        let (result_type, reply_with_value) = self.result_type_with_value();

        let await_token = self.is_async().then(|| quote!(.await));
        let unwrap_token = self.unwrap_result.then(|| quote!(.unwrap()));

        let handle_token = if reply_with_value {
            quote! {
                let command_reply: CommandReply< #result_type > = self.#handler_ident(#(#handler_params),*)#await_token #unwrap_token.into();
                let (result, value) = command_reply.to_tuple();
            }
        } else {
            quote! {
                let result = self.#handler_ident(#(#handler_params),*)#await_token #unwrap_token;
                let value = 0u128;
            }
        };

        let payable_check = if !self.payable {
            quote! {
                #[cfg(target_arch = "wasm32")]
                if #sails_path::gstd::msg::value() > 0 {
                   core::panic!("Method accepts no value");
                }
            }
        } else {
            quote!()
        };

        quote! {
            let (__encode_reply, #(#handler_params,)*) : (bool, #(#handler_types,)*) = #sails_path::alloy_sol_types::SolValue::abi_decode_params(input).ok()?;
            #payable_check
            #handle_token
            let output = if __encode_reply {
                // encode MessageId and result if passed `encode_reply`
                let message_id = #sails_path::alloy_primitives::B256::new(#sails_path::gstd::Syscall::message_id().into_bytes());
                #sails_path::alloy_sol_types::SolValue::abi_encode_sequence(&(message_id, result,))
            } else {
                #sails_path::alloy_sol_types::SolValue::abi_encode_sequence(&(result,))
            };
            return Some((output, value, __encode_reply));

        }
    }
}
