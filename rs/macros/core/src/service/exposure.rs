use super::*;
use proc_macro2::TokenStream;
use quote::quote;

impl ServiceBuilder<'_> {
    pub(super) fn exposure_struct(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;
        let route_ident = &self.route_ident;
        let inner_ident = &self.inner_ident;

        let check_asyncness_impl = self.check_asyncness_impl();

        let exposure_with_events = self.events_type.map(|events_type| {
            quote! {
                impl<T: #sails_path::meta::ServiceMeta> #sails_path::gstd::services::ExposureWithEvents for #exposure_ident<T> {
                    type Events = #events_type;
                }
            }
        });

        quote! {
            pub struct #exposure_ident<T> {
                #route_ident : &'static [u8],
                #inner_ident : T,
            }

            impl<T: #sails_path::meta::ServiceMeta> #sails_path::gstd::services::Exposure for #exposure_ident<T> {
                fn route(&self) -> &'static [u8] {
                    self. #route_ident
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

    pub(super) fn exposure_impl(&self, service_impl: &ItemImpl) -> TokenStream {
        let sails_path = self.sails_path;
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

        let exposure_funcs = &service_impl.items;

        quote! {
            #( #exposure_allow_attrs )*
            impl #generics #exposure_ident< #service_type_path > #service_type_constraints {
                #( #exposure_funcs )*

                pub fn check_asyncness(&self, input: &[u8]) -> Option<bool> {
                    <Self as #sails_path::gstd::services::Exposure>::check_asyncness(input)
                }

                #try_handle_impl

                #try_handle_solidity_impl

                #exposure_emit_event_impls

                #exposure_emit_eth_impls
            }
        }
    }

    pub(super) fn try_handle_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let inner_ident = &self.inner_ident;
        let input_ident = &self.input_ident;
        let impl_inner = |is_async: bool| {
            let (name_ident, asyncness, await_token) = if is_async {
                (
                    quote!(try_handle_async),
                    Some(quote!(async)),
                    Some(quote!(.await)),
                )
            } else {
                (quote!(try_handle), None, None)
            };

            let invocation_dispatches = self.service_handlers.iter().filter_map(|fn_builder| {
                if is_async == fn_builder.is_async() {
                    Some(fn_builder.try_handle_branch_impl(&self.meta_module_ident, input_ident))
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
                        if base_services #idx_token .expose(self.route) . #name_ident(#input_ident, result_handler) #await_token.is_some() {
                            return Some(());
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
                pub #asyncness fn #name_ident(mut self, #input_ident : &[u8], result_handler: fn(&[u8], u128)) -> Option<()> {
                    use #sails_path::gstd::InvocationIo;
                    use #sails_path::gstd::services::{Service, Exposure};
                    #( #invocation_dispatches )*
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

    pub(super) fn service_trait_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();

        let route_ident = &self.route_ident;
        let inner_ident = &self.inner_ident;

        quote!(
            impl #generics #sails_path::gstd::services::Service for #service_type_path #service_type_constraints {
                type Exposure = #exposure_ident<Self>;

                fn expose(self, #route_ident : &'static [u8]) -> Self::Exposure {
                    Self::Exposure {
                        #route_ident ,
                        #inner_ident : self,
                    }
                }
            }
        )
    }

    fn check_asyncness_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let input_ident = &self.input_ident;

        // Here `T` is Service Type
        let service_asyncness_check = quote! {
            if !T::ASYNC {
                // Return early if service is not async.
                // If there's no matching route for the input,
                // the error will be returned on the `try_handle` call.
                return Some(false);
            }
        };

        let asyncness_checks = self.service_handlers.iter().map(|fn_builder| {
            fn_builder.check_asyncness_branch_impl(&self.meta_module_ident, input_ident)
        });

        let base_services_asyncness_checks = self.base_types.iter().map(|base_type| {
            let path_wo_lifetimes = shared::remove_lifetimes(base_type);
            quote! {
                if let Some(is_async) = <<#path_wo_lifetimes as Service>::Exposure as Exposure>::check_asyncness(#input_ident) {
                    return Some(is_async);
                }
            }
        });

        quote! {
            fn check_asyncness(#input_ident : &[u8]) -> Option<bool> {
                use #sails_path::gstd::InvocationIo;
                use #sails_path::gstd::services::{Service, Exposure};

                #service_asyncness_check

                #( #asyncness_checks )*

                #( #base_services_asyncness_checks )*

                None
            }
        }
    }
}
