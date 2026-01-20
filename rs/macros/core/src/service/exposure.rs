use super::*;
use proc_macro2::TokenStream;
use quote::quote;

impl ServiceBuilder<'_> {
    pub(super) fn exposure_struct(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;
        let route_idx_ident = &self.route_idx_ident;
        let inner_ident = &self.inner_ident;

        let check_asyncness_impl = self.check_asyncness_impl();

        let exposure_with_events = self.events_type.map(|events_type| {
            quote! {
                impl<T: #sails_path::meta::ServiceMeta> #sails_path::gstd::services::ExposureWithEvents for #exposure_ident<T> {
                    type Events = #events_type;
                }
            }
        });

        let exposure_into_base = self.exposure_into_base_impl();

        quote! {
            pub struct #exposure_ident<T> {
                #route_idx_ident : u8,
                #inner_ident : T,
            }

            impl<T: #sails_path::meta::ServiceMeta> #sails_path::gstd::services::Exposure for #exposure_ident<T> {
                fn interface_id() -> #sails_path::meta::InterfaceId {
                    <T as #sails_path::meta::Identifiable>::INTERFACE_ID
                }

                fn route_idx(&self) -> u8 {
                    self. #route_idx_ident
                }

                #check_asyncness_impl
            }

            #exposure_with_events

            impl<T> core::ops::Deref for #exposure_ident<T> {
                type Target = T;

                fn deref(&self) -> &Self::Target {
                    &self. #inner_ident
                }
            }

            impl<T> core::ops::DerefMut for #exposure_ident<T> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self. #inner_ident
                }
            }

            #exposure_into_base
        }
    }

    fn exposure_emit_event_impls(&self) -> Option<TokenStream> {
        let sails_path = self.sails_path;

        self.events_type.map(|events_type| {
            quote! {
                pub fn emit_event(&self, event: #events_type) -> #sails_path::errors::Result<()> {
                    use #sails_path::gstd::services::ExposureWithEvents;

                    self.emitter().emit_event(event)
                }
            }
        })
    }

    pub(super) fn generate_decode_and_handle(
        &self,
        fn_builder: &FnBuilder,
        await_token: &Option<TokenStream>,
    ) -> TokenStream {
        let sails_path = self.sails_path;
        let meta_module_ident = &self.meta_module_ident;
        let handler_func_ident = fn_builder.ident;
        let params_struct_ident = &fn_builder.params_struct_ident;
        let handler_func_params = fn_builder
            .params_idents()
            .iter()
            .map(|ident| quote!(request.#ident));

        let (result_type, reply_with_value) = fn_builder.result_type_with_value();
        let unwrap_token = fn_builder.unwrap_result.then(|| quote!(.unwrap()));

        let handle_token = if reply_with_value {
            quote! {
                let command_reply: CommandReply< #result_type > = self.#handler_func_ident(#(#handler_func_params),*)#await_token #unwrap_token.into();
                let (result, value) = command_reply.to_tuple();
            }
        } else {
            quote! {
                let result = self.#handler_func_ident(#(#handler_func_params),*)#await_token #unwrap_token;
                let value = 0u128;
            }
        };

        let result_type_static = fn_builder.result_type_with_static_lifetime();

        quote! {
            let request: #meta_module_ident::#params_struct_ident = #sails_path::scale_codec::Decode::decode(&mut input)
                .expect("Failed to decode params");
            #handle_token
            if ! #sails_path::gstd::is_empty_tuple::<#result_type_static>() {
                <#meta_module_ident::#params_struct_ident as #sails_path::gstd::InvocationIo>::with_optimized_encode(
                    &result,
                    self.route_idx,
                    |encoded_result| result_handler(encoded_result, value),
                );
            }
            return Some(());
        }
    }

    pub(super) fn generate_dispatch_impl(
        &self,
        params: DispatchParams,
        handler_gen: impl Fn(&FnBuilder, &Option<TokenStream>) -> TokenStream,
        base_call_gen: impl Fn(TokenStream, &Option<TokenStream>, &Ident) -> TokenStream,
    ) -> TokenStream {
        let sails_path = self.sails_path;
        let inner_ident = &self.inner_ident;
        let DispatchParams {
            is_async,
            method_name_ident,
            method_sig,
            extra_imports,
            metadata_type,
        } = params;

        let (async_kw, await_token) = if is_async {
            (Some(quote!(async)), Some(quote!(.await)))
        } else {
            (None, None)
        };

        let mut regular_dispatches = Vec::new();

        for fn_builder in &self.service_handlers {
            if is_async != fn_builder.is_async() {
                continue;
            }

            let entry_id = fn_builder.entry_id;
            let decode_and_handle = handler_gen(fn_builder, &await_token);

            regular_dispatches.push(quote! {
                #entry_id => {
                    #decode_and_handle
                }
            });
        }

        let base_invocation = if self.base_types.is_empty() {
            None
        } else {
            let base_exposure_invocations = self.base_types.iter().enumerate().map(|(idx, base_type)| {
                let idx_literal = Literal::usize_unsuffixed(idx);
                let base_call = base_call_gen(quote!(base_service), &await_token, method_name_ident);
                quote! {
                    if #sails_path::meta::service_has_interface_id(&<#metadata_type as #sails_path::meta::ServiceMeta>::BASE_SERVICES[#idx_literal], interface_id) {
                        let base_service: #base_type = self.#inner_ident.into();
                        return #base_call;
                    }
                }
            });

            Some(quote! {
                #( #base_exposure_invocations )*
            })
        };

        quote! {
            pub #async_kw fn #method_name_ident #method_sig {
                #extra_imports

                // Then check own methods
                if interface_id == <#metadata_type as #sails_path::meta::Identifiable>::INTERFACE_ID {
                    match entry_id {
                        #( #regular_dispatches )*
                        _ => None,
                    }
                } else {
                    #base_invocation
                    None
                }
            }
        }
    }

    fn generate_handle_method(&self, is_async: bool) -> TokenStream {
        let sails_path = self.sails_path;
        let service_type_path = self.type_path;
        let method_ident = Ident::new(
            if is_async {
                "try_handle_async"
            } else {
                "try_handle"
            },
            Span::call_site(),
        );

        let method_sig = quote! {
            (
                mut self,
                interface_id: #sails_path::meta::InterfaceId,
                entry_id: u16,
                mut input: &[u8],
                result_handler: fn(&[u8], u128)
            ) -> Option<()>
        };

        let extra_imports = quote! {
            use #sails_path::gstd::{InvocationIo, CommandReply};
        };

        let metadata_type = quote!(self::#service_type_path);

        let params = DispatchParams {
            is_async,
            method_name_ident: &method_ident,
            method_sig: &method_sig,
            extra_imports: &extra_imports,
            metadata_type: &metadata_type,
        };

        self.generate_dispatch_impl(
            params,
            |fn_builder, await_token| self.generate_decode_and_handle(fn_builder, await_token),
            |base_service_token, await_token, method_name| {
                quote! {
                    #sails_path::gstd::services::Service::expose(#base_service_token, self.route_idx) . #method_name(interface_id, entry_id, input, result_handler) #await_token
                }
            }
        )
    }

    pub(super) fn exposure_impl(&self) -> TokenStream {
        let exposure_ident = &self.exposure_ident;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();

        // We propagate only known attributes as we don't know the consequences of unknown ones
        let exposure_allow_attrs = self
            .service_impl
            .attrs
            .iter()
            .filter(|attr| matches!(attr.path().get_ident(), Some(ident) if ident == "allow"));

        let try_handle_impl = self.try_handle_impl();
        // ethexe
        let try_handle_solidity_impl = self.try_handle_solidity_impl();

        let exposure_emit_event_impls = self.exposure_emit_event_impls();
        let exposure_emit_eth_impls = self.exposure_emit_eth_impls();

        let exposure_funcs = &self.service_impl.items;

        quote! {
            #( #exposure_allow_attrs )*
            impl #generics #exposure_ident< #service_type_path > #service_type_constraints {
                #( #exposure_funcs )*

                #try_handle_impl

                #try_handle_solidity_impl

                #exposure_emit_event_impls

                #exposure_emit_eth_impls
            }
        }
    }

    pub(super) fn try_handle_impl(&self) -> TokenStream {
        let sync_impl = self.generate_handle_method(false);
        let async_impl = self.generate_handle_method(true);

        quote! {
            #sync_impl
            #async_impl
        }
    }

    pub(super) fn service_trait_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();

        let route_idx_ident = &self.route_idx_ident;
        let inner_ident = &self.inner_ident;

        quote!(
            impl #generics #sails_path::gstd::services::Service for #service_type_path #service_type_constraints {
                type Exposure = #exposure_ident<Self>;

                fn expose(self, #route_idx_ident : u8) -> Self::Exposure {
                    Self::Exposure {
                        #route_idx_ident ,
                        #inner_ident : self,
                    }
                }
            }
        )
    }

    fn check_asyncness_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;

        // Here `T` is Service Type
        let service_asyncness_check = quote! {
            if !T::ASYNC {
                // Return early if service is not async.
                // If there's no matching route for the input,
                // the error will be returned on the `try_handle` call.
                return Some(false);
            }
        };

        // Generate match arms for each handler's entry_id
        let asyncness_checks = self.service_handlers.iter().map(|fn_builder| {
            let entry_id = fn_builder.entry_id;
            let is_async = fn_builder.is_async();
            quote! {
                #entry_id => Some(#is_async),
            }
        });

        let base_services_asyncness_checks = self.base_types.iter().enumerate().map(|(idx, base_type)| {
            let path_wo_lifetimes = shared::remove_lifetimes(base_type);
            let idx_literal = Literal::usize_unsuffixed(idx);
            quote! {
                if #sails_path::meta::service_has_interface_id(&T::BASE_SERVICES[#idx_literal], interface_id) {
                    return <<#path_wo_lifetimes as #sails_path::gstd::services::Service>::Exposure as #sails_path::gstd::services::Exposure>::check_asyncness(interface_id, entry_id);
                }
            }
        });

        quote! {
            fn check_asyncness(interface_id: #sails_path::meta::InterfaceId, entry_id: u16) -> Option<bool> {
                #service_asyncness_check

                if interface_id == <T as #sails_path::meta::Identifiable>::INTERFACE_ID {
                    match entry_id {
                        #( #asyncness_checks )*
                        _ => None,
                    }
                } else {
                    #( #base_services_asyncness_checks )*
                    None
                }
            }
        }
    }

    /// Implements `Into< TBase > for Exposure<T>`
    pub(super) fn exposure_into_base_impl(&self) -> TokenStream {
        let exposure_ident = &self.exposure_ident;
        let inner_ident = &self.inner_ident;

        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();

        let into_impl = self.base_types.iter().map(|base_type| {
            quote! {
                #[allow(clippy::from_over_into)]
                impl #generics Into< #base_type > for #exposure_ident< #service_type_path > #service_type_constraints {
                    fn into(self) -> #base_type {
                        self. #inner_ident .into()
                    }
                }
            }
        });

        quote! {
            #( #into_impl )*
        }
    }
}