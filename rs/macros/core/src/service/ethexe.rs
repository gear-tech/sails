use super::*;
use proc_macro2::TokenStream;
use quote::quote;

impl ServiceBuilder<'_> {
    pub(super) fn service_signature_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let service_type_path = self.type_path;
        let generics = &self.generics;
        let service_type_constraints = self.type_constraints.as_ref();
        let service_method_routes = self
            .service_handlers
            .iter()
            .map(|fn_builder| fn_builder.sol_handler_signature());

        let combined_methods = if self.base_types.is_empty() {
            quote! {
                &[#( #service_method_routes )*]
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
                    <#sails_path::solidity::MethodRoute>,
                    &[#( #service_method_routes )*],
                    #( #base_methods ),*
                )
            }
        };

        quote! {
            impl #generics #sails_path::solidity::ServiceSignature for #service_type_path #service_type_constraints {
                const METHODS: &'static [#sails_path::solidity::MethodRoute] = #combined_methods;
            }
        }
    }

    pub(super) fn try_handle_solidity_impl(&self, base_ident: &Ident) -> TokenStream {
        let service_method_branches = self
            .service_handlers
            .iter()
            .map(|fn_builder| fn_builder.sol_try_handle_branch_impl());
        let base_types_try_handle = self.base_types.iter().enumerate().map(|(idx, _)| {
            let idx = Literal::usize_unsuffixed(idx);
            quote! {
                if let Some((output, value)) = self. #base_ident . #idx .try_handle_solidity(method, input).await {
                    return Some((output, value));
                }
            }
        });

        quote! {
            pub async fn try_handle_solidity(
                &mut self,
                method: &[u8],
                input: &[u8],
            ) -> Option<(Vec<u8>, u128)> {
                #( #service_method_branches )*
                #( #base_types_try_handle )*
                None
            }
        }
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
        let sails_path = self.sails_path;
        let handler_route_bytes = self.encoded_route.as_slice();
        let invocation = self.sol_invocation_func();

        quote! {
            if method == &[ #(#handler_route_bytes),* ] {
                #invocation
                return Some((#sails_path::alloy_sol_types::SolValue::abi_encode(&result), value));
            }
        }
    }

    /// Generates code for encode/decode parameters and fn invocation
    /// ```rust
    /// let (_, p1, p2): (u128, u32, String) = sails_rs::alloy_sol_types::SolValue::abi_decode_params(input, false).ok()?;
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

        quote! {
            let (_, #(#handler_params,)*) : (u128, #(#handler_types,)*) = #sails_path::alloy_sol_types::SolValue::abi_decode_params(input, false).ok()?;
            #handle_token
        }
    }
}
