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
            .map(|fn_builder| fn_builder.sol_handler_signature(Some(service_type_path)));

        let combined_methods = if self.base_types.is_empty() {
            quote! {
                &[#( #service_method_expo, )*]
            }
        } else {
            let base_methods = self.sorted_base_indices.iter().map(|&idx| {
                let path_wo_lifetimes = shared::remove_lifetimes(&self.base_types[idx]);
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
        let impl_method = |is_async: bool| {
            let method_ident = Ident::new(if is_async { "try_handle_solidity_async" } else { "try_handle_solidity" }, Span::call_site());
            let sails_path = self.sails_path;
            
            let method_sig = quote! {
                (
                    mut self,
                    interface_id: #sails_path::meta::InterfaceId,
                    entry_id: u16,
                    input: &[u8],
                ) -> Option<(#sails_path::Vec<u8>, u128, bool)>
            };

            self.generate_dispatch_impl(
                is_async,
                &method_ident,
                method_sig,
                |fn_builder, _| self.generate_sol_decode_and_handle(fn_builder),
                |idx_token, await_token, method_name| {
                    quote! {
                        if let Some(result) = base_services #idx_token .expose(self.route_idx) . #method_name (interface_id, entry_id, input) #await_token {
                            return Some(result);
                        }
                    }
                }
            )
        };

        let sync_impl = impl_method(false);
        let async_impl = impl_method(true);

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

    fn generate_sol_decode_and_handle(&self, fn_builder: &FnBuilder) -> TokenStream {
        let sails_path = self.sails_path;
        let handler_ident = fn_builder.ident;
        let handler_params = fn_builder.params_idents();
        let sol_types = fn_builder.params_types().iter().map(|t| {
            quote! {
                << #t as #sails_path::alloy_sol_types::SolValue >::SolType as #sails_path::alloy_sol_types::SolType>::RustType
            }
        });
        let handler_params_into = fn_builder.params_idents().iter().map(|p| {
            quote! {
                #p.into()
            }
        });

        let (result_type, reply_with_value) = fn_builder.result_type_with_value();

        let await_token = fn_builder.is_async().then(|| quote!(.await));
        let unwrap_token = fn_builder.unwrap_result.then(|| quote!(.unwrap()));

        let handle_token = if reply_with_value {
            quote! {
                let command_reply: CommandReply< #result_type > = self.#handler_ident(#(#handler_params_into),*)#await_token #unwrap_token.into();
                let (result, value) = command_reply.to_tuple();
            }
        } else {
            quote! {
                let result = self.#handler_ident(#(#handler_params_into),*)#await_token #unwrap_token;
                let value = 0u128;
            }
        };

        let payable_check = fn_builder.payable_check();

        quote! {
            let (__encode_reply, #(#handler_params,)*) : (bool, #(#sol_types,)*) = #sails_path::alloy_sol_types::SolValue::abi_decode_params(input).ok()?;
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
